#![doc(include = "item.md")]

// Imports
use crate::game::{
	card::property::{self, ArrowColor, Effect, EffectCondition},
	util::{
		array_split, array_split_mut,
		null_ascii_string::{self, NullAsciiString},
	},
	Bytes,
};
use byteorder::{ByteOrder, LittleEndian};

/// An item card
///
/// Contains all information about each item card stored in the [`Card Table`](crate::game::card::table::Table)
#[derive(PartialEq, Eq, Clone, Hash, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Item {
	/// The item's name
	///
	/// An ascii string with 20 characters at most
	pub name: ascii::AsciiString,

	/// The effect's description.
	///
	/// The description is split along 4 lines, each
	/// being an ascii string with 20 characters at most.
	pub effect_description: [ascii::AsciiString; 4],

	/// The effect's arrow color
	#[serde(default)]
	pub effect_arrow_color: Option<ArrowColor>,

	/// The effect's conditions
	#[serde(default)]
	pub effect_conditions: [Option<EffectCondition>; 2],

	/// The effects
	#[serde(default)]
	pub effects: [Option<Effect>; 3],

	/// Unknown field at `0x15`
	pub unknown_15: u32,
}

/// Error type for [`Bytes::from_bytes`]
#[derive(PartialEq, Eq, Clone, Copy, Debug, thiserror::Error)]
pub enum FromBytesError {
	/// Unable to read the digimon name
	#[error("Unable to read the digimon name")]
	Name(#[source] null_ascii_string::ReadError),

	/// Unable to read the first support effect description
	#[error("Unable to read the first line of the effect description")]
	EffectDescriptionFirst(#[source] null_ascii_string::ReadError),

	/// Unable to read the second support effect description
	#[error("Unable to read the second line of the effect description")]
	EffectDescriptionSecond(#[source] null_ascii_string::ReadError),

	/// Unable to read the third support effect description
	#[error("Unable to read the third line of the effect description")]
	EffectDescriptionThird(#[source] null_ascii_string::ReadError),

	/// Unable to read the fourth support effect description
	#[error("Unable to read the fourth line of the effect description")]
	EffectDescriptionFourth(#[source] null_ascii_string::ReadError),

	/// An unknown effect arrow color was found
	#[error("Unknown effect arrow color found")]
	ArrowColor(#[source] property::arrow_color::FromBytesError),

	/// Unable to read the first effect condition
	#[error("Unable to read the first effect condition")]
	EffectConditionFirst(#[source] property::effect_condition::FromBytesError),

	/// Unable to read the second effect condition
	#[error("Unable to read the second effect condition")]
	EffectConditionSecond(#[source] property::effect_condition::FromBytesError),

	/// Unable to read the first effect
	#[error("Unable to read the first effect")]
	EffectFirst(#[source] property::effect::FromBytesError),

	/// Unable to read the second effect
	#[error("Unable to read the second effect")]
	EffectSecond(#[source] property::effect::FromBytesError),

	/// Unable to read the third effect
	#[error("Unable to read the third effect")]
	EffectThird(#[source] property::effect::FromBytesError),
}

/// Error type for [`Bytes::to_bytes`]
#[derive(PartialEq, Eq, Clone, Copy, Debug, thiserror::Error)]
pub enum ToBytesError {
	/// Unable to write the digimon name
	#[error("Unable to write the digimon name")]
	Name(#[source] null_ascii_string::WriteError),

	/// Unable to write the first support effect description
	#[error("Unable to write the first line of the effect description")]
	EffectDescriptionFirst(#[source] null_ascii_string::WriteError),

	/// Unable to write the second support effect description
	#[error("Unable to write the second line of the effect description")]
	EffectDescriptionSecond(#[source] null_ascii_string::WriteError),

	/// Unable to write the third support effect description
	#[error("Unable to write the third line of the effect description")]
	EffectDescriptionThird(#[source] null_ascii_string::WriteError),

	/// Unable to write the fourth support effect description
	#[error("Unable to write the fourth line of the effect description")]
	EffectDescriptionFourth(#[source] null_ascii_string::WriteError),

	/// Unable to write the first effect
	#[error("Unable to write the first effect")]
	EffectFirst(#[source] property::effect::ToBytesError),

	/// Unable to write the second effect
	#[error("Unable to write the second effect")]
	EffectSecond(#[source] property::effect::ToBytesError),

	/// Unable to write the third effect
	#[error("Unable to write the third effect")]
	EffectThird(#[source] property::effect::ToBytesError),
}

impl Bytes for Item {
	type ByteArray = [u8; 0xde];
	type FromError = FromBytesError;
	type ToError = ToBytesError;

	fn from_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::FromError> {
		// Split bytes
		let bytes = array_split!(bytes,
			name                : [0x15],
			unknown_15          : [0x4],
			condition_first     : [0x20],
			condition_second    : [0x20],
			effect_first        : [0x10],
			effect_second       : [0x10],
			effect_third        : [0x10],
			effect_arrow_color  : 1,
			effect_description_0: [0x15],
			effect_description_1: [0x15],
			effect_description_2: [0x15],
			effect_description_3: [0x15],
		);

		// And return the struct
		Ok(Self {
			name: bytes.name.read_string().map_err(FromBytesError::Name)?.to_ascii_string(),

			// Effects
			effect_conditions: [
				Option::<EffectCondition>::from_bytes(bytes.condition_first).map_err(FromBytesError::EffectConditionFirst)?,
				Option::<EffectCondition>::from_bytes(bytes.condition_second).map_err(FromBytesError::EffectConditionSecond)?,
			],

			effects: [
				Option::<Effect>::from_bytes(bytes.effect_first).map_err(FromBytesError::EffectFirst)?,
				Option::<Effect>::from_bytes(bytes.effect_second).map_err(FromBytesError::EffectSecond)?,
				Option::<Effect>::from_bytes(bytes.effect_third).map_err(FromBytesError::EffectThird)?,
			],

			effect_arrow_color: Option::<ArrowColor>::from_bytes(bytes.effect_arrow_color).map_err(FromBytesError::ArrowColor)?,

			effect_description: [
				bytes
					.effect_description_0
					.read_string()
					.map_err(FromBytesError::EffectDescriptionFirst)?
					.to_ascii_string(),
				bytes
					.effect_description_1
					.read_string()
					.map_err(FromBytesError::EffectDescriptionSecond)?
					.to_ascii_string(),
				bytes
					.effect_description_2
					.read_string()
					.map_err(FromBytesError::EffectDescriptionThird)?
					.to_ascii_string(),
				bytes
					.effect_description_3
					.read_string()
					.map_err(FromBytesError::EffectDescriptionFourth)?
					.to_ascii_string(),
			],

			// Unknown
			unknown_15: LittleEndian::read_u32(bytes.unknown_15),
		})
	}

	fn to_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::ToError> {
		// Split bytes
		let bytes = array_split_mut!(bytes,
			name                : [0x15],
			unknown_15          : [0x4],
			condition_first     : [0x20],
			condition_second    : [0x20],
			effect_first        : [0x10],
			effect_second       : [0x10],
			effect_third        : [0x10],
			effect_arrow_color  : 1,
			effect_description_0: [0x15],
			effect_description_1: [0x15],
			effect_description_2: [0x15],
			effect_description_3: [0x15],
		);

		// Name
		bytes.name.write_string(&self.name).map_err(ToBytesError::Name)?;

		// Effects
		self.effect_conditions[0].to_bytes(bytes.condition_first).into_ok();
		self.effect_conditions[1].to_bytes(bytes.condition_second).into_ok();

		self.effects[0].to_bytes(bytes.effect_first).map_err(ToBytesError::EffectFirst)?;
		self.effects[1].to_bytes(bytes.effect_second).map_err(ToBytesError::EffectSecond)?;
		self.effects[2].to_bytes(bytes.effect_third).map_err(ToBytesError::EffectThird)?;

		Option::<ArrowColor>::to_bytes(&self.effect_arrow_color, bytes.effect_arrow_color).into_ok();

		bytes
			.effect_description_0
			.write_string(&self.effect_description[0])
			.map_err(ToBytesError::EffectDescriptionFirst)?;
		bytes
			.effect_description_1
			.write_string(&self.effect_description[1])
			.map_err(ToBytesError::EffectDescriptionSecond)?;
		bytes
			.effect_description_2
			.write_string(&self.effect_description[2])
			.map_err(ToBytesError::EffectDescriptionThird)?;
		bytes
			.effect_description_3
			.write_string(&self.effect_description[3])
			.map_err(ToBytesError::EffectDescriptionFourth)?;

		// Unknown
		LittleEndian::write_u32(bytes.unknown_15, self.unknown_15);

		// Return Ok
		Ok(())
	}
}
