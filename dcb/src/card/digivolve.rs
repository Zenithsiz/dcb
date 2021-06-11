#![doc = include_str!("digivolve.md")]

// Imports
use crate::card::property::{digivolve_effect, DigivolveEffect};
use dcb_bytes::Bytes;
use dcb_util::{
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

/// Error type for [`Bytes::deserialize_bytes`](dcb_bytes::Bytes::deserialize_bytes)
#[derive(PartialEq, Eq, Clone, Copy, Debug, thiserror::Error)]
pub enum DeserializeBytesError {
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
	Effect(#[source] digivolve_effect::DeserializeBytesError),
}

impl Bytes for Digivolve {
	type ByteArray = [u8; 0x6c];
	type DeserializeError = DeserializeBytesError;
	type SerializeError = !;

	fn deserialize_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::DeserializeError> {
		// Split bytes
		let bytes = dcb_util::array_split!(bytes,
			name                : [0x15],
			effect              : [0x3],
			effect_description_0: [0x15],
			effect_description_1: [0x15],
			effect_description_2: [0x15],
			effect_description_3: [0x15],
		);

		Ok(Self {
			// Name
			name: bytes.name.read_string().map_err(DeserializeBytesError::Name)?,

			// Effect
			effect_description: [
				bytes
					.effect_description_0
					.read_string()
					.map_err(DeserializeBytesError::EffectDescription1)?,
				bytes
					.effect_description_1
					.read_string()
					.map_err(DeserializeBytesError::EffectDescription2)?,
				bytes
					.effect_description_2
					.read_string()
					.map_err(DeserializeBytesError::EffectDescription3)?,
				bytes
					.effect_description_3
					.read_string()
					.map_err(DeserializeBytesError::EffectDescription4)?,
			],

			// Unknown
			effect: DigivolveEffect::deserialize_bytes(bytes.effect).map_err(DeserializeBytesError::Effect)?,
		})
	}

	fn serialize_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::SerializeError> {
		// Split bytes
		let bytes = dcb_util::array_split_mut!(bytes,
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
		self.effect.serialize_bytes(bytes.effect).into_ok();

		// Return Ok
		Ok(())
	}
}
