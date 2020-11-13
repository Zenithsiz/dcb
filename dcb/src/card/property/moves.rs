#![doc(include = "move.md")]

// Modules
#[cfg(test)]
mod test;

// Imports
use crate::{Validatable, Validation};
use byteorder::{ByteOrder, LittleEndian};
use dcb_bytes::Bytes;
use dcb_util::{
	array_split, array_split_mut,
	null_ascii_string::{self, NullAsciiString},
	AsciiStrArr,
};

/// A digimon's move
#[derive(PartialEq, Eq, Clone, Hash, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Move {
	/// The move's name
	name: AsciiStrArr<0x15>,

	/// The move's power
	power: u16,

	/// The unknown data
	unknown: u32,
}

/// Error type for [`Bytes::from_bytes`](dcb_bytes::Bytes::from_bytes)
#[derive(PartialEq, Eq, Clone, Copy, Debug, thiserror::Error)]
pub enum FromBytesError {
	/// Unable to read the move name
	#[error("Unable to read the move name")]
	Name(#[source] null_ascii_string::ReadError),
}

impl Bytes for Move {
	type ByteArray = [u8; 0x1c];
	type FromError = FromBytesError;
	type ToError = !;

	fn from_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::FromError> {
		// Get all byte arrays we need
		let bytes = array_split!(bytes,
			power  : [0x2],
			unknown: [0x4],
			name   : [0x16],
		);

		// Return the move
		Ok(Self {
			name:    bytes.name.read_string().map_err(FromBytesError::Name)?,
			power:   LittleEndian::read_u16(bytes.power),
			unknown: LittleEndian::read_u32(bytes.unknown),
		})
	}

	fn to_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::ToError> {
		// Get all byte arrays we need
		let bytes = array_split_mut!(bytes,
			power  : [0x2],
			unknown: [0x4],
			name   : [0x16],
		);

		// Write the name
		bytes.name.write_string(&self.name);

		// Then write the power and the unknown
		LittleEndian::write_u16(bytes.power, self.power);
		LittleEndian::write_u32(bytes.unknown, self.unknown);

		// And return Ok
		Ok(())
	}
}

impl Validatable for Move {
	type Error = ValidationError;
	type Warning = ValidationWarning;

	fn validate(&self) -> Validation<Self::Error, Self::Warning> {
		// Create the initial validation
		let mut validation = Validation::new();

		// If the power isn't a multiple of 10, warn, as we don't know how the game handles
		// powers that aren't multiples of 10.
		// TODO: Verify if the game can handle non-multiple of 10 powers.
		if self.power % 10 != 0 {
			validation.emit_warning(ValidationWarning::PowerMultiple10);
		}

		// And return the validation
		validation
	}
}

/// All warnings for [`Move`] validation
#[derive(PartialEq, Eq, Clone, Debug, thiserror::Error)]
pub enum ValidationWarning {
	/// Power is not a multiple of 10
	#[error("Power is not a multiple of 10.")]
	PowerMultiple10,
}

/// All errors for [`Move`] validation
#[derive(PartialEq, Eq, Clone, Debug, thiserror::Error)]
pub enum ValidationError {
	/// Name length
	#[error("Name is too long. Must be at most 21 characters")]
	NameTooLong,
}
