#![doc(include = "digivolve.md")]

// Imports
use crate::card::property::{digivolve_effect, DigivolveEffect};
use dcb_bytes::Bytes;
use dcb_util::{
	array_split, array_split_mut,
	null_ascii_string::{self, NullAsciiString},
	AsciiStrArr,
};

/// A digivolve card
///
/// Contains all information about each digivolve card stored in the [`Card Table`](crate::card::table::Table)
#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Digivolve {
	/// The item's name
	pub name: AsciiStrArr<0x14>,

	/// The effect's description.
	///
	/// The description is split along 4 lines
	pub effect_description: [AsciiStrArr<0x14>; 4],

	/// Effect
	pub effect: DigivolveEffect,
}

/// Error type for [`Bytes::from_bytes`](dcb_bytes::Bytes::from_bytes)
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

	/// Unable to parse the effect
	#[error("Unable to parse the effect")]
	Effect(#[source] digivolve_effect::FromBytesError),
}

impl Bytes for Digivolve {
	type ByteArray = [u8; 0x6c];
	type FromError = FromBytesError;
	type ToError = !;

	fn from_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::FromError> {
		// Split bytes
		let bytes = array_split!(bytes,
			name                : [0x15],
			effect              : [0x3],
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
				bytes
					.effect_description_0
					.read_string()
					.map_err(FromBytesError::EffectDescription1)?,
				bytes
					.effect_description_1
					.read_string()
					.map_err(FromBytesError::EffectDescription2)?,
				bytes
					.effect_description_2
					.read_string()
					.map_err(FromBytesError::EffectDescription3)?,
				bytes
					.effect_description_3
					.read_string()
					.map_err(FromBytesError::EffectDescription4)?,
			],

			// Unknown
			effect: DigivolveEffect::from_bytes(bytes.effect).map_err(FromBytesError::Effect)?,
		})
	}

	fn to_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::ToError> {
		// Split bytes
		let bytes = array_split_mut!(bytes,
			name                : [0x15],
			effect              : [0x3],
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
		self.effect.to_bytes(bytes.effect).into_ok();

		// Return Ok
		Ok(())
	}
}
