//! Directives

// Imports
use super::{InstFmt, InstSize};
use crate::exe::{DataType, Pos};
use ascii::{AsciiChar, AsciiStr};
use dcb_util::NextFromBytes;

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

/// Error type for [`Directive::decode_with_data`]
#[derive(Debug, thiserror::Error)]
pub enum DecodeWithDataError {
	/// Missing bytes
	#[error("Missing bytes")]
	MissingBytes,

	/// Value wasn't aligned
	#[error("Value was not aligned")]
	NotAligned,

	/// Read position was offset from data
	#[error("Cannot read value offset")]
	Offset,

	/// String had invalid characters
	#[error("String had invalid characters")]
	StrInvalidChars(#[source] ascii::AsAsciiStrError),

	/// String had nulls within it
	#[error("String had nulls")]
	StrNullsWithin,

	/// String didn't have null terminator
	#[error("String missing null terminator")]
	StrNullTerminator,
}

impl<'a> Directive<'a> {
	/// Decodes a directive with some data
	pub fn decode_with_data(pos: Pos, bytes: &'a [u8], ty: &DataType, data_pos: Pos) -> Result<Self, DecodeWithDataError> {
		// Make sure that this function is only called when the data contains `pos`
		assert!((data_pos..data_pos + ty.size()).contains(&pos));

		// If the data isn't aligned, return None
		if !data_pos.is_aligned_to(ty.align()) {
			return Err(DecodeWithDataError::NotAligned);
		}

		// If we're not in an array, but we're not at the start of the data, return
		if !matches!(ty, DataType::Array { .. }) && pos != data_pos {
			return Err(DecodeWithDataError::Offset);
		}

		match ty {
			&DataType::AsciiStr { len } => {
				// Read the string
				let string = bytes.get(..len).ok_or(DecodeWithDataError::MissingBytes)?;
				let string = AsciiStr::from_ascii(string).map_err(DecodeWithDataError::StrInvalidChars)?;

				// If there are any nulls, return
				if string.chars().any(|ch| ch == AsciiChar::Null) {
					return Err(DecodeWithDataError::StrNullsWithin);
				}

				// Then make sure there's nulls padding the string
				let nulls_len = 4 - len % 4;
				let nulls = bytes.get(len..len + nulls_len).ok_or(DecodeWithDataError::StrNullTerminator)?;
				if !nulls.iter().all(|&ch| ch == 0) {
					return Err(DecodeWithDataError::StrNullTerminator);
				}

				// Else return the string
				Ok(Self::Ascii(string))
			},
			DataType::Word => bytes.next_u32().map(Self::Dw).ok_or(DecodeWithDataError::MissingBytes),
			DataType::HalfWord => bytes.next_u16().map(Self::Dh).ok_or(DecodeWithDataError::MissingBytes),
			DataType::Byte => bytes.next_u8().map(Self::Db).ok_or(DecodeWithDataError::MissingBytes),
			DataType::Array { ty, .. } => {
				// Get the index we're on in the array.
				let offset = pos.offset_from(data_pos);
				let idx = offset / ty.size();
				let next_data_pos = data_pos + idx * ty.size();
				Self::decode_with_data(pos, bytes, ty, next_data_pos)
			},
		}
	}

	/// Decodes a directive
	#[must_use]
	pub fn decode(pos: Pos, bytes: &'a [u8]) -> Option<Self> {
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
