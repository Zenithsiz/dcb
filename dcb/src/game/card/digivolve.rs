#![doc(include = "digivolve.md")]

// Imports
use crate::{
	util::{
		array_split, array_split_mut,
		null_ascii_string::{self, NullAsciiString},
	},
	AsciiStrArr,
};
use dcb_bytes::Bytes;

// TODO: Remove these
/// Name alias for [`Digimon`]
type NameString = AsciiStrArr<0x14>;

/// Effect description alias for [`Digimon`]
type EffectDescriptionString = AsciiStrArr<0x14>;

/// A digivolve card
///
/// Contains all information about each digivolve card stored in the [`Card Table`](crate::game::card::table::Table)
#[derive(PartialEq, Eq, Clone, Hash, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Digivolve {
	/// The item's name
	pub name: NameString,

	/// The effect's description.
	///
	/// The description is split along 4 lines
	pub effect_description: [EffectDescriptionString; 4],

	/// Unknown field at `0x15`
	pub unknown_15: [u8; 3],
}

/// Error type for [`Bytes::from_bytes`]
#[derive(PartialEq, Eq, Clone, Copy, Debug, thiserror::Error)]
pub enum FromBytesError {
	/// Unable to read the digimon name
	#[error("Unable to read the digimon name")]
	Name(#[source] null_ascii_string::ReadError),

	/// Unable to read the first support effect description
	#[error("Unable to read the first line of the effect description")]
	EffectDescription1(#[source] null_ascii_string::ReadError),

	/// Unable to read the second support effect description
	#[error("Unable to read the second line of the effect description")]
	EffectDescription2(#[source] null_ascii_string::ReadError),

	/// Unable to read the third support effect description
	#[error("Unable to read the third line of the effect description")]
	EffectDescription3(#[source] null_ascii_string::ReadError),

	/// Unable to read the fourth support effect description
	#[error("Unable to read the fourth line of the effect description")]
	EffectDescription4(#[source] null_ascii_string::ReadError),
}

impl Bytes for Digivolve {
	type ByteArray = [u8; 0x6c];
	type FromError = FromBytesError;
	type ToError = !;

	fn from_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::FromError> {
		// Split bytes
		let bytes = array_split!(bytes,
			name                : [0x15],
			unknown_15          : [0x3],
			effect_description_0: [0x15],
			effect_description_1: [0x15],
			effect_description_2: [0x15],
			effect_description_3: [0x15],
		);

		Ok(Self {
			// Name
			name: bytes.name.read_string().map_err(FromBytesError::Name)?,

			// Effect
			effect_description: [
				bytes.effect_description_0.read_string().map_err(FromBytesError::EffectDescription1)?,
				bytes.effect_description_1.read_string().map_err(FromBytesError::EffectDescription2)?,
				bytes.effect_description_2.read_string().map_err(FromBytesError::EffectDescription3)?,
				bytes.effect_description_3.read_string().map_err(FromBytesError::EffectDescription4)?,
			],

			// Unknown
			unknown_15: *bytes.unknown_15,
		})
	}

	fn to_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::ToError> {
		// Split bytes
		let bytes = array_split_mut!(bytes,
			name                : [0x15],
			unknown_15          : [0x3],
			effect_description_0: [0x15],
			effect_description_1: [0x15],
			effect_description_2: [0x15],
			effect_description_3: [0x15],
		);

		// Name
		bytes.name.write_string(&self.name);

		// Effects
		bytes.effect_description_0.write_string(&self.effect_description[0]);
		bytes.effect_description_1.write_string(&self.effect_description[1]);
		bytes.effect_description_2.write_string(&self.effect_description[2]);
		bytes.effect_description_3.write_string(&self.effect_description[3]);

		// Unknown
		*bytes.unknown_15 = self.unknown_15;

		// Return Ok
		Ok(())
	}
}
