#![doc(include = "item.md")]

// Imports
use crate::card::property::{
	self, ArrowColor, Effect, EffectCondition, MaybeArrowColor, MaybeEffect, MaybeEffectCondition,
};
use byteorder::{ByteOrder, LittleEndian};
use dcb_bytes::Bytes;
use dcb_util::{
	array_split, array_split_mut,
	null_ascii_string::{self, NullAsciiString},
	AsciiStrArr,
};
use ref_cast::RefCast;

/// An item card
///
/// Contains all information about each item card stored in the [`Card Table`](crate::card::table::Table)
#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Item {
	/// The item's name
	///
	/// An ascii string with 20 characters at most
	pub name: AsciiStrArr<0x14>,

	/// The effect's description.
	///
	/// The description is split along 4 lines, each
	/// being an ascii string with 20 characters at most.
	pub effect_description: [AsciiStrArr<0x14>; 4],

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

	/// An unknown effect arrow color was found
	#[error("Unknown effect arrow color found")]
	ArrowColor(#[source] property::arrow_color::DeserializeBytesError),

	/// Unable to read the first effect condition
	#[error("Unable to read the first effect condition")]
	EffectConditionFirst(#[source] property::effect_condition::DeserializeBytesError),

	/// Unable to read the second effect condition
	#[error("Unable to read the second effect condition")]
	EffectConditionSecond(#[source] property::effect_condition::DeserializeBytesError),

	/// Unable to read the first effect
	#[error("Unable to read the first effect")]
	EffectFirst(#[source] property::effect::DeserializeBytesError),

	/// Unable to read the second effect
	#[error("Unable to read the second effect")]
	EffectSecond(#[source] property::effect::DeserializeBytesError),

	/// Unable to read the third effect
	#[error("Unable to read the third effect")]
	EffectThird(#[source] property::effect::DeserializeBytesError),
}

/// Error type for [`Bytes::serialize_bytes`](dcb_bytes::Bytes::serialize_bytes)
#[derive(PartialEq, Eq, Clone, Copy, Debug, thiserror::Error)]
#[allow(clippy::pub_enum_variant_names)] // This is a general error, not a specific effect error
pub enum SerializeBytesError {
	/// Unable to write the first effect
	#[error("Unable to write the first effect")]
	EffectFirst(#[source] property::effect::SerializeBytesError),

	/// Unable to write the second effect
	#[error("Unable to write the second effect")]
	EffectSecond(#[source] property::effect::SerializeBytesError),

	/// Unable to write the third effect
	#[error("Unable to write the third effect")]
	EffectThird(#[source] property::effect::SerializeBytesError),
}

impl Bytes for Item {
	type ByteArray = [u8; 0xde];
	type DeserializeError = DeserializeBytesError;
	type SerializeError = SerializeBytesError;

	fn deserialize_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::DeserializeError> {
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
			name: bytes.name.read_string().map_err(DeserializeBytesError::Name)?,

			// Effects
			effect_conditions: [
				MaybeEffectCondition::deserialize_bytes(bytes.condition_first)
					.map_err(DeserializeBytesError::EffectConditionFirst)?
					.into(),
				MaybeEffectCondition::deserialize_bytes(bytes.condition_second)
					.map_err(DeserializeBytesError::EffectConditionSecond)?
					.into(),
			],

			effects: [
				MaybeEffect::deserialize_bytes(bytes.effect_first)
					.map_err(DeserializeBytesError::EffectFirst)?
					.into(),
				MaybeEffect::deserialize_bytes(bytes.effect_second)
					.map_err(DeserializeBytesError::EffectSecond)?
					.into(),
				MaybeEffect::deserialize_bytes(bytes.effect_third)
					.map_err(DeserializeBytesError::EffectThird)?
					.into(),
			],

			effect_arrow_color: MaybeArrowColor::deserialize_bytes(bytes.effect_arrow_color)
				.map_err(DeserializeBytesError::ArrowColor)?
				.into(),

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
			unknown_15: LittleEndian::read_u32(bytes.unknown_15),
		})
	}

	fn serialize_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::SerializeError> {
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
		bytes.name.write_string(&self.name);

		// Effects
		MaybeEffectCondition::ref_cast(&self.effect_conditions[0])
			.serialize_bytes(bytes.condition_first)
			.into_ok();
		MaybeEffectCondition::ref_cast(&self.effect_conditions[1])
			.serialize_bytes(bytes.condition_second)
			.into_ok();

		MaybeEffect::ref_cast(&self.effects[0])
			.serialize_bytes(bytes.effect_first)
			.map_err(SerializeBytesError::EffectFirst)?;
		MaybeEffect::ref_cast(&self.effects[1])
			.serialize_bytes(bytes.effect_second)
			.map_err(SerializeBytesError::EffectSecond)?;
		MaybeEffect::ref_cast(&self.effects[2])
			.serialize_bytes(bytes.effect_third)
			.map_err(SerializeBytesError::EffectThird)?;

		MaybeArrowColor::ref_cast(&self.effect_arrow_color)
			.serialize_bytes(bytes.effect_arrow_color)
			.into_ok();

		bytes.effect_description_0.write_string(&self.effect_description[0]);
		bytes.effect_description_1.write_string(&self.effect_description[1]);
		bytes.effect_description_2.write_string(&self.effect_description[2]);
		bytes.effect_description_3.write_string(&self.effect_description[3]);

		// Unknown
		LittleEndian::write_u32(bytes.unknown_15, self.unknown_15);

		// Return Ok
		Ok(())
	}
}
