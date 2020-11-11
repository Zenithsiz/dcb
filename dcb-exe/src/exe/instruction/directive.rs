//! Directives

// Imports
use super::{FromRawIter, Instruction, Raw};
use crate::exe::Pos;
use ascii::{AsciiChar, AsciiStr, AsciiString};
use int_conv::Split;
use smallvec::{smallvec, SmallVec};
use std::ops::{
	Bound::{self, Excluded, Included, Unbounded},
	RangeBounds,
};
use AsciiChar::Null;

/// A directive
#[derive(PartialEq, Eq, Clone, Debug)]
#[derive(derive_more::Display)]
pub enum Directive {
	/// Write word
	#[display(fmt = "dw {_0:#x}")]
	Dw(u32),

	/// Write half-word
	#[display(fmt = "dh {_0:#x}")]
	Dh(u16),

	/// Write byte
	#[display(fmt = "db {_0:#x}")]
	Db(u8),

	/// Ascii string
	#[display(fmt = ".ascii {_0:?}")]
	Ascii(AsciiString),
}

/// A force decode range
pub struct ForceDecodeRange {
	/// Start bound
	start: Bound<Pos>,

	/// End bound
	end: Bound<Pos>,

	/// Decoding kind
	kind: ForceDecodeKind,
}

impl RangeBounds<Pos> for ForceDecodeRange {
	fn start_bound(&self) -> Bound<&Pos> {
		match self.start {
			Included(ref start) => Included(start),
			Excluded(ref start) => Excluded(start),
			Unbounded => Unbounded,
		}
	}

	fn end_bound(&self) -> Bound<&Pos> {
		match self.end {
			Included(ref end) => Included(end),
			Excluded(ref end) => Excluded(end),
			Unbounded => Unbounded,
		}
	}
}

/// Force decode range kind
pub enum ForceDecodeKind {
	/// Single Word
	W,

	/// Two half-words
	HH,

	/// Half-word followed by bytes
	HBB,

	/// Bytes followed by half-word
	BBH,

	/// Bytes
	BBBB,
}

impl Directive {
	/// Positions that should be force decoded using a specific variant.
	pub const FORCE_DECODE_RANGES: &'static [ForceDecodeRange] = &[
		ForceDecodeRange {
			start: Included(Pos(0x80010000)),
			end:   Excluded(Pos(0x80010008)),
			kind:  ForceDecodeKind::W,
		},
		ForceDecodeRange {
			start: Included(Pos(0x8006fa20)),
			end:   Excluded(Pos(0x8006fa24)),
			kind:  ForceDecodeKind::HH,
		},
		ForceDecodeRange {
			start: Included(Instruction::CODE_END),
			end:   Unbounded,
			kind:  ForceDecodeKind::W,
		},
	];

	/// Returns the size of this instruction
	#[must_use]
	pub fn size(&self) -> u32 {
		#[allow(clippy::as_conversions, clippy::cast_possible_truncation)] // Our length will always fit into a `u32`.
		match self {
			Self::Dw(_) => 4,
			Self::Dh(_) => 2,
			Self::Db(_) => 1,
			// Round ascii strings' len up to the
			// nearest word.
			Self::Ascii(ascii) => {
				let len = ascii.len() as u32;
				len + 4 - (len % 4)
			},
		}
	}
}


/// Helper function to check if a string has null and if everything after the first
/// null is also null (or if there were no nulls).
fn check_nulls<S: AsRef<AsciiStr>>(s: S) -> (S, usize, bool) {
	let null_idx = s
		.as_ref()
		.as_slice()
		.iter()
		.position(|&ch| ch == Null)
		.unwrap_or_else(|| s.as_ref().len());
	#[allow(clippy::indexing_slicing)] // `null_idx <= len`
	let uniform_null = s.as_ref()[null_idx..].chars().all(|ch| ch == Null);
	(s, null_idx, uniform_null)
}

impl FromRawIter for Directive {
	//type Decoded = Option<(Pos, Self)>;
	// Note: We return at most 4 directives.
	type Decoded = SmallVec<[(Pos, Self); 4]>;

	fn decode<I: Iterator<Item = Raw> + Clone>(iter: &mut I) -> Self::Decoded {
		// Get the first raw
		let raw = match iter.next() {
			Some(raw) => raw,
			None => return smallvec![],
		};

		// If we're past all the code, there are no more strings,
		// so just decode a `dw`.
		// Note: We're working in big endian when returning these.
		if let Some(ForceDecodeRange { kind, .. }) = Self::FORCE_DECODE_RANGES.iter().find(|range| range.contains(&raw.pos)) {
			return match kind {
				ForceDecodeKind::W => smallvec![(raw.pos, Self::Dw(raw.repr))],
				ForceDecodeKind::HH => {
					let (lo, hi) = raw.repr.lo_hi();
					smallvec![(raw.pos, Self::Dh(hi)), (raw.pos + 2, Self::Dh(lo))]
				},
				ForceDecodeKind::HBB => {
					let (lo, hi) = raw.repr.lo_hi();
					let (lo_lo, lo_hi) = lo.lo_hi();
					smallvec![(raw.pos, Self::Dh(hi)), (raw.pos + 2, Self::Db(lo_hi)), (raw.pos + 3, Self::Db(lo_lo))]
				},
				ForceDecodeKind::BBH => {
					let (lo, hi) = raw.repr.lo_hi();
					let (hi_lo, hi_hi) = hi.lo_hi();
					smallvec![(raw.pos, Self::Db(hi_hi)), (raw.pos + 1, Self::Db(hi_lo)), (raw.pos + 2, Self::Dh(lo))]
				},
				ForceDecodeKind::BBBB => {
					let (lo, hi) = raw.repr.lo_hi();
					let (lo_lo, lo_hi) = lo.lo_hi();
					let (hi_lo, hi_hi) = hi.lo_hi();
					smallvec![
						(raw.pos, Self::Db(hi_hi)),
						(raw.pos + 1, Self::Db(hi_lo)),
						(raw.pos + 2, Self::Db(lo_hi)),
						(raw.pos + 3, Self::Db(lo_lo))
					]
				},
			};
		}

		// Try to get an ascii string from the raw and check for nulls
		match AsciiString::from_ascii(raw.repr.to_ne_bytes()).map(check_nulls) {
			// If we got a string with at least 1 non-null, but
			// at least 1 null and uniformly null, return just it
			Ok((mut ascii_string, null_idx @ 1..=3, true)) => {
				ascii_string.truncate(null_idx);
				smallvec![(raw.pos, Self::Ascii(ascii_string))]
			},

			// If we got a string without any nulls, keep
			// filling the string until we find one.
			Ok((mut ascii_string, 4, true)) => {
				let ascii_string = loop {
					let mut cur_iter = iter.clone();
					match cur_iter.next() {
						// If we don't have a next character, return the string as-is
						// Note: No need to update the iterator, it returned `None`.
						None => break ascii_string,

						// Else try to get it as a string and check for nulls
						Some(next_raw) => match AsciiStr::from_ascii(&next_raw.repr.to_ne_bytes()).map(check_nulls) {
							// If we got it and it wasn't null, update the iterator, add it and continue
							Ok((new_ascii_str, 4, _)) => {
								*iter = cur_iter;
								ascii_string.push_str(new_ascii_str);
							},

							// If we got it, but there was a uniform null, update the iterator,
							// add the non-null parts and return.
							#[allow(clippy::indexing_slicing)] // `null_idx < len`
							Ok((new_ascii_str, null_idx, true)) => {
								*iter = cur_iter;
								ascii_string.push_str(&new_ascii_str[..null_idx]);
								break ascii_string;
							},

							// If we didn't get it or it was a non-uniform null, return the string we have so far
							// Note: We don't update the iterator, as we want to leave
							//       the next value to `dw`.
							Err(_) | Ok((_, _, false)) => break ascii_string,
						},
					}
				};

				smallvec![(raw.pos, Self::Ascii(ascii_string))]
			},

			// Else if it was full null, non-uniformly null or non-ascii,
			// just return a normal word.
			_ => smallvec![(raw.pos, Self::Dw(raw.repr))],
		}
	}
}
