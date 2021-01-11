//! Directives

// Imports
//use super::{FromRawIter, Instruction, Raw};
use super::{Inst, InstFmt, InstSize};
use crate::exe::Pos;
use ascii::{AsciiChar, AsciiStr};
use dcb_util::NextFromBytes;
use std::ops::{
	Bound::{self, Excluded, Included, Unbounded},
	RangeBounds,
};

/// A directive
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Directive {
	/// Write word
	Dw(u32),

	/// Write half-word
	Dh(u16),

	/// Write byte
	Db(u8),

	/// Ascii string
	Ascii {
		/// String length
		len: usize,
	},
}

/// A force decode range
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
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
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum ForceDecodeKind {
	/// Single Word
	Word,

	/// Half word
	HalfWord,

	/// Bytes
	Byte,
}

impl Directive {
	/// Positions that should be force decoded using a specific variant.
	// TODO: Get this at run-time via a file.
	pub const FORCE_DECODE_RANGES: &'static [ForceDecodeRange] = &[
		ForceDecodeRange {
			start: Included(Pos(0x80010000)),
			end:   Excluded(Pos(0x80010008)),
			kind:  ForceDecodeKind::Word,
		},
		ForceDecodeRange {
			start: Included(Pos(0x8006fa20)),
			end:   Excluded(Pos(0x8006fa24)),
			kind:  ForceDecodeKind::HalfWord,
		},
		ForceDecodeRange {
			start: Included(Inst::CODE_END),
			end:   Unbounded,
			kind:  ForceDecodeKind::Word,
		},
	];
}

impl Directive {
	/// Decodes a directive
	#[must_use]
	pub fn decode(pos: Pos, bytes: &[u8]) -> Option<Self> {
		// Check if we need to force decode it
		if let Some(ForceDecodeRange { kind, .. }) = Self::FORCE_DECODE_RANGES.iter().find(|range| range.contains(&pos)) {
			#[rustfmt::skip]
			return match kind {
				ForceDecodeKind::Word     => bytes.next_u32().map(Self::Dw),
				ForceDecodeKind::HalfWord => bytes.next_u16().map(Self::Dh),
				ForceDecodeKind::Byte     => bytes.next_u8 ().map(Self::Db),
			};
		}

		// TODO: Respect alignment

		// Else try to get a string
		if let Some(len) = self::read_ascii_until_null(bytes) {
			return Some(Self::Ascii { len });
		}

		// Else try to read a `u32`
		if let Some(value) = bytes.next_u32() {
			return Some(Self::Dw(value));
		}

		// Else read a single byte
		bytes.next_u8().map(Self::Db)
	}
}

impl InstSize for Directive {
	fn size(&self) -> usize {
		match self {
			Self::Dw(_) => 4,
			Self::Dh(_) => 2,
			Self::Db(_) => 1,
			// Round ascii strings' len up to the
			// nearest word (or one after if exactly 1 word).
			Self::Ascii { len } => len + 4 - (len % 4),
		}
	}
}

impl InstFmt for Directive {
	fn mnemonic(&self) -> &'static str {
		match self {
			Self::Dw(_) => "dw",
			Self::Dh(_) => "dh",
			Self::Db(_) => "db",
			Self::Ascii { .. } => ".ascii",
		}
	}

	fn fmt(&self, pos: Pos, bytes: &[u8], f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let mnemonic = self.mnemonic();
		match self {
			Self::Dw(value) => write!(f, "{mnemonic} {value:#x}"),
			Self::Dh(value) => write!(f, "{mnemonic} {value:#x}"),
			Self::Db(value) => write!(f, "{mnemonic} {value:#x}"),
			&Self::Ascii { len } => {
				let pos = pos.as_mem_idx();
				let string = &bytes[pos..pos + len];
				let string = AsciiStr::from_ascii(string).expect("Ascii string was invalid").as_str();
				write!(f, "{mnemonic} \"{}\"", string.escape_debug())
			},
		}
	}
}

/// Reads an ascii string from a byte slice until null, aligned to a word
#[allow(clippy::as_conversions, clippy::cast_possible_truncation)] // Our length will always fit into a `u32`.
fn read_ascii_until_null(bytes: &[u8]) -> Option<usize> {
	// For each set of 4 bytes in the input
	for (bytes, cur_size) in bytes.array_chunks::<4>().zip((0..).step_by(4)) {
		// If the bytes aren't all ascii, return
		if !bytes.iter().all(|&ch| AsciiChar::from_ascii(ch).is_ok()) {
			return None;
		}

		// Else check if we got any nulls
		// Note: In order to return, after the first null, we must have
		//       all nulls until the end of the word.
		#[allow(clippy::match_same_arms)] // We can't change the order of the arms.
		return match bytes {
			// If we got all nulls, as long as we aren't empty, return the string
			[0, 0, 0, 0] => match cur_size {
				0 => None,
				_ => Some(cur_size + 4),
			},
			[0, _, _, _] => None,
			[_, 0, 0, 0] => Some(cur_size + 4),
			[_, 0, _, _] => None,
			[_, _, 0, 0] => Some(cur_size + 4),
			[_, _, 0, _] => None,
			[_, _, _, 0] => Some(cur_size + 4),

			_ => continue,
		};
	}
	None
}
