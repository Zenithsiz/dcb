//! Directives

// Imports
use super::{InstFmt, InstSize};
use crate::exe::Pos;
use ascii::AsciiStr;
use dcb_util::NextFromBytes;
use std::ops::{
	Bound::{self, Excluded, Included, Unbounded},
	RangeBounds,
};

/// A directive
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Directive<'a> {
	/// Write word
	Dw(u32),

	/// Write half-word
	Dh(u16),

	/// Write byte
	Db(u8),

	/// Ascii string
	Ascii(&'a AsciiStr),
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

impl<'a> Directive<'a> {
	/*
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
	*/
}

impl<'a> Directive<'a> {
	/// Decodes a directive
	#[must_use]
	pub fn decode(pos: Pos, bytes: &'a [u8]) -> Option<Self> {
		/*
		// Check if we need to force decode it
		if let Some(ForceDecodeRange { kind, .. }) = Self::FORCE_DECODE_RANGES.iter().find(|range| range.contains(&pos)) {
			#[rustfmt::skip]
			return match kind {
				ForceDecodeKind::Word     => bytes.next_u32().map(Self::Dw),
				ForceDecodeKind::HalfWord => bytes.next_u16().map(Self::Dh),
				ForceDecodeKind::Byte     => bytes.next_u8 ().map(Self::Db),
			};
		}
		*/

		// If we're not half-word aligned, read a byte
		if !pos.is_half_word_aligned() {
			return Some(Self::Db(bytes.next_u8()?));
		}

		// If we're not word aligned, read a half-word
		if !pos.is_word_aligned() {
			return Some(Self::Dh(bytes.next_u16()?));
		}

		// Else try to get a string, since we're word aligned
		if let Some(string) = self::read_ascii_until_null(bytes) {
			return Some(Self::Ascii(string));
		}

		// Else try to read a word
		if let Some(value) = bytes.next_u32() {
			return Some(Self::Dw(value));
		}

		// Else try to read a half-word
		if let Some(value) = bytes.next_u16() {
			return Some(Self::Dh(value));
		}

		// Else read a single byte
		bytes.next_u8().map(Self::Db)
	}
}

impl<'a> InstSize for Directive<'a> {
	fn size(&self) -> usize {
		match self {
			Self::Dw(_) => 4,
			Self::Dh(_) => 2,
			Self::Db(_) => 1,
			// Round ascii strings' len up to the
			// nearest word (or one after if exactly 1 word).
			Self::Ascii(string) => string.len() + 4 - (string.len() % 4),
		}
	}
}

impl<'a> InstFmt for Directive<'a> {
	fn fmt(&self, _pos: Pos, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Self::Dw(value) => write!(f, "dw {value:#x}"),
			Self::Dh(value) => write!(f, "dh {value:#x}"),
			Self::Db(value) => write!(f, "db {value:#x}"),
			Self::Ascii(string) => write!(f, ".ascii \"{}\"", string.as_str().escape_debug()),
		}
	}
}

/// Reads an ascii string from a byte slice until null, aligned to a word
#[allow(clippy::as_conversions, clippy::cast_possible_truncation)] // Our length will always fit into a `u32`.
fn read_ascii_until_null(bytes: &[u8]) -> Option<&AsciiStr> {
	// For each word in the input
	for (word, cur_size) in bytes.array_chunks::<4>().zip((0..).step_by(4)) {
		// If the bytes aren't all ascii, return
		if AsciiStr::from_ascii(word).is_err() {
			return None;
		}

		// Else check if we got any nulls, to finish the string.
		// Note: In order to return, after the first null, we must have
		//       all nulls until the end of the word.
		#[allow(clippy::match_same_arms)] // We can't change the order of the arms.
		let len = match word {
			// If we got all nulls, as long as we aren't empty, return the string
			[0, 0, 0, 0] => match cur_size {
				0 => return None,
				_ => cur_size,
			},
			[0, _, _, _] => return None,
			[_, 0, 0, 0] => cur_size + 1,
			[_, 0, _, _] => return None,
			[_, _, 0, 0] => cur_size + 2,
			[_, _, 0, _] => return None,
			[_, _, _, 0] => cur_size + 3,

			_ => continue,
		};

		// Then build the string
		let string = AsciiStr::from_ascii(&bytes[..len]).expect("Checked the string was valid");
		return Some(string);
	}
	None
}
