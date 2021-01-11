//! Data types

// Imports
use ::std::fmt;

use ::serde::de::{Deserialize, Deserializer};
use serde::de::Visitor;

/// Data types
#[derive(PartialEq, Eq, Clone, Hash, Debug)]
#[derive(derive_more::Display)]
pub enum DataType {
	/// Ascii string
	#[display(fmt = "str")]
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
		len: usize,
	},
}

impl DataType {
	/// Returns the size of this data kind
	#[must_use]
	pub fn size(&self) -> usize {
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
		// If it starts with '[', read it as an array
		if let Some(s) = s.strip_prefix('[') {
			// Find the first ';' from the end to split.
			let s = s.strip_suffix(']').ok_or(FromStrError::MissingArraySuffix)?;
			let (ty, len) = s
				.char_indices()
				.rev()
				.find_map(|(pos, c)| c.eq(&';').then(|| s.split_at(pos)))
				.ok_or(FromStrError::MissingArraySep)?;

			// Ignore the leading ';' on the second.
			let len = &len[1..];

			// Trim both strings
			let ty = ty.trim();
			let len = len.trim();

			let ty = Self::from_str(ty).map_err(|err| FromStrError::InvalidArrayTy(Box::new(err)))?;
			let ty = Box::new(ty);
			let len = self::parse_usize(len).map_err(|err| FromStrError::InvalidArrayLen { len: len.to_owned(), err })?;

			return Ok(Self::Array { ty, len });
		}

		// Else check the type
		match s {
			"AsciiChar" => Ok(Self::AsciiChar),
			"u8" => Ok(Self::Byte),
			"u16" => Ok(Self::HalfWord),
			"u32" => Ok(Self::Word),
			_ => Err(FromStrError::UnknownTy { ty: s.to_owned() }),
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
pub fn parse_usize(s: &str) -> Result<usize, std::num::ParseIntError> {
	let (s, base) = match s.trim().as_bytes() {
		[b'0', b'x', rest @ ..] => (rest, 16),
		[b'0', b'o', rest @ ..] => (rest, 8),
		[b'0', b'b', rest @ ..] => (rest, 2),
		s => (s, 10),
	};
	let s = std::str::from_utf8(s).expect("Failed to convert `str` -> `[u8]` -> `str`");
	usize::from_str_radix(s, base)
}
