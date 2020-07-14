#![doc(include = "digivolve.md")]

// Imports
use crate::game::{util, Bytes};

/// A digivolve card
///
/// Contains all information about each digivolve card stored in the [`Card Table`](crate::game::card::table::Table)
#[derive(PartialEq, Eq, Clone, Hash, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Digivolve {
	/// The item's name
	///
	/// An ascii string with 20 characters at most
	pub name: ascii::AsciiString,

	/// The effect's description.
	///
	/// The description is split along 4 lines, each
	/// being an ascii string with 20 characters at most.
	pub effect_description: [ascii::AsciiString; 4],

	/// Unknown field at `0x15`
	pub unknown_15: [u8; 3],
}

/// Error type for [`Bytes::from_bytes`]
#[derive(PartialEq, Eq, Clone, Copy, Debug, thiserror::Error)]
pub enum FromBytesError {
	/// Unable to read the digimon name
	#[error("Unable to read the digimon name")]
	Name(#[source] util::ReadNullAsciiStringError),

	/// Unable to read the first support effect description
	#[error("Unable to read the first line of the effect description")]
	EffectDescriptionFirst(#[source] util::ReadNullAsciiStringError),

	/// Unable to read the second support effect description
	#[error("Unable to read the second line of the effect description")]
	EffectDescriptionSecond(#[source] util::ReadNullAsciiStringError),

	/// Unable to read the third support effect description
	#[error("Unable to read the third line of the effect description")]
	EffectDescriptionThird(#[source] util::ReadNullAsciiStringError),

	/// Unable to read the fourth support effect description
	#[error("Unable to read the fourth line of the effect description")]
	EffectDescriptionFourth(#[source] util::ReadNullAsciiStringError),
}

/// Error type for [`Bytes::to_bytes`]
#[derive(PartialEq, Eq, Clone, Copy, Debug, thiserror::Error)]
pub enum ToBytesError {
	/// Unable to write the digimon name
	#[error("Unable to write the digimon name")]
	Name(#[source] util::WriteNullAsciiStringError),

	/// Unable to write the first support effect description
	#[error("Unable to write the first line of the effect description")]
	EffectDescriptionFirst(#[source] util::WriteNullAsciiStringError),

	/// Unable to write the second support effect description
	#[error("Unable to write the second line of the effect description")]
	EffectDescriptionSecond(#[source] util::WriteNullAsciiStringError),

	/// Unable to write the third support effect description
	#[error("Unable to write the third line of the effect description")]
	EffectDescriptionThird(#[source] util::WriteNullAsciiStringError),

	/// Unable to write the fourth support effect description
	#[error("Unable to write the fourth line of the effect description")]
	EffectDescriptionFourth(#[source] util::WriteNullAsciiStringError),
}

impl Bytes for Digivolve {
	type ByteArray = [u8; 0x6c];
	type FromError = FromBytesError;
	type ToError = ToBytesError;

	fn from_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::FromError> {
		// Split bytes
		let bytes = util::array_split!(bytes,
			name                : [0x15],
			unknown_15          : [0x3],
			effect_description_0: [0x15],
			effect_description_1: [0x15],
			effect_description_2: [0x15],
			effect_description_3: [0x15],
		);

		Ok(Self {
			// Name
			name: util::read_null_ascii_string(bytes.name).map_err(FromBytesError::Name)?.to_ascii_string(),

			// Effect
			effect_description: [
				util::read_null_ascii_string(bytes.effect_description_0)
					.map_err(FromBytesError::EffectDescriptionFirst)?
					.to_ascii_string(),
				util::read_null_ascii_string(bytes.effect_description_1)
					.map_err(FromBytesError::EffectDescriptionSecond)?
					.to_ascii_string(),
				util::read_null_ascii_string(bytes.effect_description_2)
					.map_err(FromBytesError::EffectDescriptionThird)?
					.to_ascii_string(),
				util::read_null_ascii_string(bytes.effect_description_3)
					.map_err(FromBytesError::EffectDescriptionFourth)?
					.to_ascii_string(),
			],

			// Unknown
			unknown_15: *bytes.unknown_15,
		})
	}

	fn to_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::ToError> {
		// Split bytes
		let bytes = util::array_split_mut!(bytes,
			name                : [0x15],
			unknown_15          : [0x3],
			effect_description_0: [0x15],
			effect_description_1: [0x15],
			effect_description_2: [0x15],
			effect_description_3: [0x15],
		);

		// Name
		util::write_null_ascii_string(self.name.as_ref(), bytes.name).map_err(ToBytesError::Name)?;

		// Effects
		util::write_null_ascii_string(self.effect_description[0].as_ref(), bytes.effect_description_0)
			.map_err(ToBytesError::EffectDescriptionFirst)?;
		util::write_null_ascii_string(self.effect_description[1].as_ref(), bytes.effect_description_1)
			.map_err(ToBytesError::EffectDescriptionSecond)?;
		util::write_null_ascii_string(self.effect_description[2].as_ref(), bytes.effect_description_2)
			.map_err(ToBytesError::EffectDescriptionThird)?;
		util::write_null_ascii_string(self.effect_description[3].as_ref(), bytes.effect_description_3)
			.map_err(ToBytesError::EffectDescriptionFourth)?;

		// Unknown
		*bytes.unknown_15 = self.unknown_15;

		// Return Ok
		Ok(())
	}
}
