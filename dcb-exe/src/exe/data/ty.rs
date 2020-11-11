//! Data types

// Imports
use ::std::fmt;

use ::serde::de::{Deserialize, Deserializer};
use serde::de::Visitor;

/// Data types
#[derive(PartialEq, Eq, Clone, Hash, Debug)]
#[derive(derive_more::Display)]
pub enum DataType {
	/// Ascii character
	#[display(fmt = "AsciiChar")]
	AsciiChar,

	/// Word
	#[display(fmt = "u32")]
	Word,

	/// Half-word
	#[display(fmt = "u16")]
	HalfWord,

	/// Byte
	#[display(fmt = "u8")]
	Byte,

	/// Array
	#[display(fmt = "[{ty}; {len}]")]
	Array {
		/// Array type
		ty: Box<DataType>,

		/// Array length
		len: u32,
	},
}

impl DataType {
	/// Returns the size of this data kind
	#[must_use]
	pub fn size(&self) -> u32 {
		match self {
			Self::Word => 4,
			Self::HalfWord => 2,
			Self::Byte | Self::AsciiChar => 1,
			Self::Array { ty, len } => len * ty.size(),
		}
	}
}

/// Error for [`FromStr`](std::str::FromStr) impl.
#[derive(Debug, thiserror::Error)]
pub enum FromStrError {
	/// Missing ']' in array.
	#[error("Missing ']' after '[...'")]
	MissingArraySuffix,

	/// Missing array separator, ';'.
	#[error("Missing ';' in array '[...;...]'")]
	MissingArraySep,

	/// Invalid array type
	#[error("Invalid array type")]
	InvalidArrayTy(#[source] Box<Self>),

	/// Invalid array length
	#[error("Invalid array length '{len}'")]
	InvalidArrayLen {
		/// Invalid length
		len: String,

		/// Underlying error
		#[source]
		err: std::num::ParseIntError,
	},

	/// Unknown type
	#[error("Unknown type '{ty}'")]
	UnknownTy {
		/// The unknown type.
		ty: String,
	},
}

impl std::str::FromStr for DataType {
	type Err = FromStrError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		// Strip any whitespace.
		let s = s.trim();

		// If it starts with '[', read it as an array
		if let Some(s) = s.strip_prefix('[') {
			let s = s.strip_suffix(']').ok_or(FromStrError::MissingArraySuffix)?;
			// Note: We find the first ';' from the end to split.
			let (ty, len) = s
				.char_indices()
				.rev()
				.find_map(|(pos, c)| c.eq(&';').then(|| s.split_at(pos)))
				.ok_or(FromStrError::MissingArraySep)?;
			// Ignore the leading ';'
			#[allow(clippy::indexing_slicing)] // This can't panic, as `pos < len.`
			let len = &len[1..];

			let ty = box Self::from_str(ty).map_err(|err| FromStrError::InvalidArrayTy(box err))?;
			let len = self::parse_u32(len).map_err(|err| FromStrError::InvalidArrayLen { len: len.to_string(), err })?;

			return Ok(Self::Array { ty, len });
		}

		// Else check the type
		match s {
			"AsciiChar" => Ok(Self::AsciiChar),
			"u8" => Ok(Self::Byte),
			"u16" => Ok(Self::HalfWord),
			"u32" => Ok(Self::Word),
			_ => Err(FromStrError::UnknownTy { ty: s.to_string() }),
		}
	}
}

impl<'de> Deserialize<'de> for DataType {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		deserializer.deserialize_str(DataTypeVisitor)
	}
}

/// Visitor
pub struct DataTypeVisitor;

impl<'de> Visitor<'de> for DataTypeVisitor {
	type Value = DataType;

	fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		formatter.write_str("a data type")
	}

	fn visit_str<E>(self, value: &str) -> Result<DataType, E>
	where
		E: serde::de::Error,
	{
		value.parse().map_err(E::custom)
	}
}

impl serde::Serialize for DataType {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		serializer.serialize_str(&self.to_string())
	}
}

/// Helper function to parse a `u32` from a string with any base.
pub fn parse_u32(s: &str) -> Result<u32, std::num::ParseIntError> {
	let (s, base) = match s.trim().as_bytes() {
		[b'0', b'x', len @ ..] => (len, 16),
		[b'0', b'o', len @ ..] => (len, 8),
		[b'0', b'b', len @ ..] => (len, 2),
		s => (s, 10),
	};
	let s = std::str::from_utf8(s).expect("Failed to convert `str` -> `[u8]` -> `str`");
	u32::from_str_radix(s, base)
}
