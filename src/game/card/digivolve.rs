//! A digivolve card
//!
//! This module contains the [`Digivolve`] struct, which describes a digivolve card.
//!
//! # Layout
//! The digivolve card has a size of `0x6c` bytes, and it's layout is the following:
//!
//! | Offset | Size | Type                | Name                      | Location               | Details                                                             |
//! |--------|------|---------------------|---------------------------|------------------------|---------------------------------------------------------------------|
//! | 0x0    | 0x15 | `[char; 0x15]`      | Name                      | `name`                 | Null-terminated                                                     |
//! | 0x15   | 0x3  | `[u8; 3]`           | Unknown                   | `unknown_15`           | Probably contains the card effect                                   |
//! | 0x8a   | 0x54 | `[[char; 0x15]; 4]` | Effect description lines  | `effect_description`   | Each line is` 0x15` bytes, split over 4 lines, each null terminated |

// Crate
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

	// Unknown
	pub unknown_15: [u8; 3],
}

/// Error type for [`Bytes::from_bytes`]
#[derive(Debug)]
#[derive(derive_more::Display, err_impl::Error)]
pub enum FromBytesError {
	/// Unable to read the digimon name
	#[display(fmt = "Unable to read the digimon name")]
	Name(#[error(source)] util::ReadNullAsciiStringError),

	/// Unable to read the first support effect description
	#[display(fmt = "Unable to read the first line of the effect description")]
	EffectDescriptionFirst(#[error(source)] util::ReadNullAsciiStringError),

	/// Unable to read the second support effect description
	#[display(fmt = "Unable to read the second line of the effect description")]
	EffectDescriptionSecond(#[error(source)] util::ReadNullAsciiStringError),

	/// Unable to read the third support effect description
	#[display(fmt = "Unable to read the third line of the effect description")]
	EffectDescriptionThird(#[error(source)] util::ReadNullAsciiStringError),

	/// Unable to read the fourth support effect description
	#[display(fmt = "Unable to read the fourth line of the effect description")]
	EffectDescriptionFourth(#[error(source)] util::ReadNullAsciiStringError),
}

/// Error type for [`Bytes::to_bytes`]
#[derive(Debug)]
#[derive(derive_more::Display, err_impl::Error)]
pub enum ToBytesError {
	/// Unable to write the digimon name
	#[display(fmt = "Unable to write the digimon name")]
	Name(#[error(source)] util::WriteNullAsciiStringError),

	/// Unable to write the first support effect description
	#[display(fmt = "Unable to write the first line of the effect description")]
	EffectDescriptionFirst(#[error(source)] util::WriteNullAsciiStringError),

	/// Unable to write the second support effect description
	#[display(fmt = "Unable to write the second line of the effect description")]
	EffectDescriptionSecond(#[error(source)] util::WriteNullAsciiStringError),

	/// Unable to write the third support effect description
	#[display(fmt = "Unable to write the third line of the effect description")]
	EffectDescriptionThird(#[error(source)] util::WriteNullAsciiStringError),

	/// Unable to write the fourth support effect description
	#[display(fmt = "Unable to write the fourth line of the effect description")]
	EffectDescriptionFourth(#[error(source)] util::WriteNullAsciiStringError),
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
