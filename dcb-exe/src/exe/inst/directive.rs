//! Directives

// Imports
//use super::{FromRawIter, Instruction, Raw};
use super::Inst;
use crate::exe::Pos;
use ascii::AsciiChar;
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
		len: u32,
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

	/// Returns the size of this directive
	#[must_use]
	pub const fn size(self) -> u32 {
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

impl Directive {
	/// Decodes a directive
	#[must_use]
	pub fn decode(pos: Pos, bytes: &[u8]) -> Option<(Self, usize)> {
		// Check if we need to force decode it
		if let Some(ForceDecodeRange { kind, .. }) = Self::FORCE_DECODE_RANGES.iter().find(|range| range.contains(&pos)) {
			#[rustfmt::skip]
			return match kind {
				ForceDecodeKind::Word     => bytes.next_u32().map(|value| (Self::Dw(value), 4)),
				ForceDecodeKind::HalfWord => bytes.next_u16().map(|value| (Self::Dh(value), 2)),
				ForceDecodeKind::Byte     => bytes.next_u8 ().map(|value| (Self::Db(value), 1)),
			};
		}

		// Else try to get a string
		if let Some((str_len, with_nulls_len)) = self::read_ascii_until_null(pos, bytes) {
			debug_assert!(with_nulls_len % 4 == 0, "Ascii string length wasn't multiple of 4");
			return Some((Self::Ascii { len: str_len }, with_nulls_len));
		}

		// Else try to read a `u32`
		if let Some(value) = bytes.next_u32() {
			return Some((Self::Dw(value), 4));
		}

		// Else read a single byte
		bytes.next_u8().map(|value| (Self::Db(value), 1))
	}
}

/// Reads an ascii string from a byte slice until null.
///
/// Will always read in multiples of a word (4 bytes), including the null.
#[allow(clippy::as_conversions, clippy::cast_possible_truncation)] // Our length will always fit into a `u32`.
fn read_ascii_until_null(pos: Pos, bytes: &[u8]) -> Option<(u32, usize)> {
	// Get the next null or invalid character
	let (idx, null) = bytes.iter().enumerate().find_map(|(idx, &byte)| match AsciiChar::from_ascii(byte) {
		Ok(AsciiChar::Null) => Some((idx, true)),
		Err(_) => Some((idx, false)),
		_ => None,
	})?;

	// If it wasn't a null or the first character was a null, return None
	if !null || idx == 0 {
		return None;
	}

	// Else make sure until the end of the word it's all nulls
	let nulls_len = 4 - ((pos.0 as usize + idx) % 4);
	let nulls = bytes.get(idx..idx + nulls_len)?;
	if !nulls.iter().all(|&byte| byte == 0) {
		return None;
	}

	// Else return both lengths
	Some((idx as u32, idx + nulls_len))
}
