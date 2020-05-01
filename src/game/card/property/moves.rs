//! A digimon's move
//!
//! This module contains the [`Move`] struct, which describes a generic move over the triangle, circle or cross.
//!
//! # Layout
//! Each move has a size of `0x1c` bytes, and it's layout is the following:
//!
//! | Offset | Size | Type                 | Name                      | Location               | Details                           |
//! |--------|------|----------------------|---------------------------|------------------------|-----------------------------------|
//! | 0x0    | 0x2  | `u16`                | Power                     | `power`                |                                   |
//! | 0x2    | 0x4  | `u32`                | Unknown                   | `unknown`              | Most likely stores animation data |
//! | 0x6    | 0x16 | `[char; 0x16]`       | Name                      | `name`                 | Null-terminated                   |

// byteorder
use byteorder::{ByteOrder, LittleEndian};

// Crate
use crate::game::{util, Bytes};

/// A digimon's move
#[derive(PartialEq, Eq, Clone, Hash, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Move {
	/// The move's name
	name: ascii::AsciiString,

	/// The move's power
	power: u16,

	/// The unknown data
	unknown: u32,
}

/// Error type for [`Bytes::from_bytes`]
#[derive(Debug, derive_more::Display, err_impl::Error)]
pub enum FromBytesError {
	/// Unable to read the move name
	#[display(fmt = "Unable to read the move name")]
	Name(#[error(source)] util::ReadNullAsciiStringError),
}

/// Error type for [`Bytes::to_bytes`]
#[derive(Debug, derive_more::Display, err_impl::Error)]
pub enum ToBytesError {
	/// Unable to write the move name
	#[display(fmt = "Unable to write the move name")]
	Name(#[error(source)] util::WriteNullAsciiStringError),
}

// Bytes
impl Bytes for Move {
	type ByteArray = [u8; 0x1c];
	type FromError = FromBytesError;
	type ToError = ToBytesError;

	fn from_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::FromError> {
		// Get all byte arrays we need
		let bytes = util::array_split!(bytes,
			power  : [0x2],
			unknown: [0x4],
			name   : [0x16],
		);

		// Return the move
		Ok(Self {
			name: util::read_null_ascii_string(bytes.name).map_err(FromBytesError::Name)?.chars().collect(),
			power: LittleEndian::read_u16(bytes.power),
			unknown: LittleEndian::read_u32(bytes.unknown),
		})
	}

	fn to_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::ToError> {
		// Get all byte arrays we need
		let bytes = util::array_split_mut!(bytes,
			power  : [0x2],
			unknown: [0x4],
			name   : [0x16],
		);

		// Write the name
		util::write_null_ascii_string(self.name.as_ref(), bytes.name).map_err(ToBytesError::Name)?;

		// Then write the power and the unknown
		LittleEndian::write_u16(bytes.power, self.power);
		LittleEndian::write_u32(bytes.unknown, self.unknown);

		// And return Ok
		Ok(())
	}
}
