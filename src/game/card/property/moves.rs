#![doc(include = "move.md")]

// Modules
#[cfg(test)]
mod test;

// byteorder
use byteorder::{ByteOrder, LittleEndian};

// Crate
use crate::game::{bytes::Validation, util, Bytes};

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
			name:    util::read_null_ascii_string(bytes.name).map_err(FromBytesError::Name)?.to_ascii_string(),
			power:   LittleEndian::read_u16(bytes.power),
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

	fn validate(&self) -> Validation {
		// Create the initial validation
		let mut validation = Validation::new();

		// If our name is longer or equal to `0x16` bytes, emit error
		if self.name.len() >= 0x16 {
			validation.add_error("Name must be at most 21 characters.");
		}

		// If the power isn't a multiple of 10, warn, as we don't know how the game handles
		// powers that aren't multiples of 10.
		// TODO: Verify if the game can handle non-multiple of 10 powers.
		if self.power % 10 != 0 {
			validation.add_warning("Powers that are not a multiple of 10 are not fully supported.");
		}

		// And return the validation
		validation
	}
}
