//! Data types

// Imports
use crate::inst::parse::line::{parse_literal, ParseLiteralError}; /* TODO: Stop importing these from here, move
                                                                    * them elsewhere */
use std::{convert::TryInto, str::FromStr};

/// Data types
#[derive(PartialEq, Eq, Clone, Hash, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(derive_more::Display)]
pub enum DataType {
	/// Ascii string
	#[display(fmt = "AsciiStr<{len}>")]
	AsciiStr {
		/// String length
		len: usize,
	},

	/// Word
	#[display(fmt = "u32")]
	#[serde(rename = "u32")]
	Word,

	/// Half-word
	#[display(fmt = "u16")]
	#[serde(rename = "u16")]
	HalfWord,

	/// Byte
	#[display(fmt = "u8")]
	#[serde(rename = "u8")]
	Byte,

	/// Array
	#[display(fmt = "Arr<{ty}, {len}>")]
	Array {
		/// Array type
		ty: Box<DataType>,

		/// Array length
		len: usize,
	},

	/// Marker
	#[display(fmt = "Marker<{len}>")]
	Marker {
		/// Byte size
		len: usize,
	},
}

/// Error for [`FromStr`] impl of [`DataType`]
#[derive(PartialEq, Clone, Debug, thiserror::Error)]
pub enum FromStrError {
	/// Leftover tokens in string
	#[error("Leftover tokens")]
	LeftoverTokens,

	/// Expected name before `<`
	#[error("Expected name before `<`")]
	NameBeforeGenerics,

	/// Unknown type
	#[error("Unknown type")]
	UnknownType,

	/// Unknown generic
	#[error("Unknown generic")]
	UnknownGeneric,

	/// Expected end of generic arguments
	#[error("Expected end of generic arguments")]
	EndGenerics,

	/// Unable to parse length
	#[error("Unable to parse length")]
	ParseLen(#[source] ParseLiteralError),

	/// Length didn't fit
	#[error("Length didn't fit")]
	LenOutOfBounds,

	/// Missing comma in arguments
	#[error("Missing comma in arguments")]
	ArgsComma,
}

// TODO: Improve this impl.
impl FromStr for DataType {
	type Err = FromStrError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		/// Parses partially a data type from string
		fn parse_partial(s: &str) -> Result<(DataType, &str), FromStrError> {
			// Read a name until a delimiter
			let (name, rest) = match s.find(|c: char| !c.is_alphanumeric()) {
				Some(idx) => (&s[..idx], &s[idx..]),
				None => (s, ""),
			};
			let rest = rest.trim_start();

			// If name is empty, return Err
			if name.is_empty() {
				return Err(FromStrError::NameBeforeGenerics);
			}

			// If we have generics, parse them
			if let Some(rest) = rest.strip_prefix('<') {
				let rest = rest.trim_start();

				// Check the generic
				return match name {
					"AsciiStr" => {
						let (len, rest) = self::parse_literal(rest).map_err(FromStrError::ParseLen)?;
						let len = len.try_into().map_err(|_| FromStrError::LenOutOfBounds)?;
						let rest = rest.trim_start().strip_prefix('>').ok_or(FromStrError::EndGenerics)?;
						Ok((DataType::AsciiStr { len }, rest))
					},
					"Arr" => {
						// Read the type
						let (ty, rest) = parse_partial(rest)?;
						let rest = rest.trim_start();

						// If there isn't a comma, return Err
						let rest = rest.strip_prefix(',').ok_or(FromStrError::ArgsComma)?.trim_start();

						let (len, rest) = self::parse_literal(rest).map_err(FromStrError::ParseLen)?;
						let len = len.try_into().map_err(|_| FromStrError::LenOutOfBounds)?;
						let rest = rest.trim_start().strip_prefix('>').ok_or(FromStrError::EndGenerics)?;
						Ok((DataType::Array { ty: Box::new(ty), len }, rest))
					},
					"Marker" => {
						let (len, rest) = self::parse_literal(rest).map_err(FromStrError::ParseLen)?;
						let len = len.try_into().map_err(|_| FromStrError::LenOutOfBounds)?;
						let rest = rest.trim_start().strip_prefix('>').ok_or(FromStrError::EndGenerics)?;
						Ok((DataType::Marker { len }, rest))
					},
					_ => Err(FromStrError::UnknownGeneric),
				};
			}

			// Else just parse the type
			match name {
				"u32" => Ok((DataType::Word, rest)),
				"u16" => Ok((DataType::HalfWord, rest)),
				"u8" => Ok((DataType::Byte, rest)),
				_ => Err(FromStrError::UnknownType),
			}
		}

		let (ty, rest) = parse_partial(s)?;
		if !rest.is_empty() {
			return Err(FromStrError::LeftoverTokens);
		}
		Ok(ty)
	}
}

impl DataType {
	/// Returns the size of this data kind
	#[must_use]
	pub fn size(&self) -> usize {
		match self {
			Self::Word => 4,
			Self::HalfWord => 2,
			Self::Byte => 1,
			// Round strings to the nearest word
			Self::AsciiStr { len } => len + 4 - (len % 4),
			Self::Array { ty, len } => len * ty.size(),
			&Self::Marker { len } => len,
		}
	}

	/// Returns the alignment of this data kind
	#[must_use]
	#[allow(clippy::match_same_arms)] // Looks better like this
	pub fn align(&self) -> usize {
		match self {
			Self::Word => 4,
			Self::HalfWord => 2,
			Self::Byte => 1,
			Self::AsciiStr { .. } => 4,
			Self::Array { ty, .. } => ty.align(),
			Self::Marker { .. } => 1,
		}
	}
}
