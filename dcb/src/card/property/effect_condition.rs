#![doc = include_str!("effect_condition.md")]

// Imports
use crate::card::property::{self, DigimonProperty, EffectConditionOperation, MaybeDigimonProperty};
use byteorder::{ByteOrder, LittleEndian};
use dcb_bytes::Bytes;
use ref_cast::RefCast;

/// A digimon's support effect condition
#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct EffectCondition {
	/// If the effect should throw a misfire when false
	pub misfire: bool,

	/// The property to compare to
	pub property_cmp: DigimonProperty,

	/// The property argument
	pub arg_property: Option<DigimonProperty>,

	/// The number argument
	pub arg_num: u16,

	/// The operation
	pub operation: EffectConditionOperation,
}

/// The error type thrown by `FromBytes`
#[derive(PartialEq, Eq, Clone, Copy, Debug, thiserror::Error)]
pub enum DeserializeBytesError {
	/// Unable to read the condition
	#[error("Unable to read the effect condition")]
	Condition(#[source] property::digimon_property::DeserializeBytesError),

	/// Unable to read a property argument
	#[error("Unable to read the property argument")]
	PropertyArgument(#[source] property::digimon_property::DeserializeBytesError),

	/// Unable to read the effect operation
	#[error("Unable to read the effect operation")]
	Operation(#[source] property::effect_condition_operation::DeserializeBytesError),
}

impl Bytes for EffectCondition {
	type ByteArray = [u8; 0x20];
	type DeserializeError = DeserializeBytesError;
	type SerializeError = !;

	fn deserialize_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::DeserializeError> {
		let bytes = dcb_util::array_split!(bytes,
			misfire     : 0x1,
			zero_0      : 0x1,
			property_cmp: 0x1,
			zero_1      : [0x5],
			arg_property: 0x1,
			zero_2      : [0xb],
			arg_num     : [0x2],
			zero_3      : [0x4],
			operation   : 1,
			zero_4      : [0x5],
		);

		// Make sure all zeros are actually zero in debug mode.
		// Except for `zero_1`, as the card `Heap of Junk` seems to
		// have the value `[0, 22, 0, 0, 0]` here for some reason, but
		// it doesn't seem necessary
		debug_assert_eq!(*bytes.zero_0, 0);
		match *bytes.zero_1 {
			[0, 22, 0, 0, 0] => {
				log::warn!("Found bytes `[0, 22, 0, 0, 0]` for effect condition `zero_1`.");
				log::info!("The previous warning should only appear for \"Heap of Junk\" in the original game file.");
			},
			_ => debug_assert_eq!(*bytes.zero_1, [0; 0x5]),
		}
		debug_assert_eq!(*bytes.zero_2, [0; 0xb]);
		debug_assert_eq!(*bytes.zero_3, [0; 0x4]);
		debug_assert_eq!(*bytes.zero_4, [0; 0x5]);

		Ok(Self {
			misfire:      (*bytes.misfire != 0),
			property_cmp: DigimonProperty::deserialize_bytes(bytes.property_cmp)
				.map_err(DeserializeBytesError::Condition)?,

			arg_property: MaybeDigimonProperty::deserialize_bytes(bytes.arg_property)
				.map_err(DeserializeBytesError::PropertyArgument)?
				.into(),

			arg_num: LittleEndian::read_u16(bytes.arg_num),

			operation: EffectConditionOperation::deserialize_bytes(bytes.operation)
				.map_err(DeserializeBytesError::Operation)?,
		})
	}

	fn serialize_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::SerializeError> {
		let bytes = dcb_util::array_split_mut!(bytes,
			misfire     : 0x1,
			zero_0      : 0x1,
			property_cmp: 0x1,
			zero_1      : [0x5],
			arg_property: 0x1,
			zero_2      : [0xb],
			arg_num     : [0x2],
			zero_3      : [0x4],
			operation   : 1,
			zero_4      : [0x5],
		);

		// Misfire
		*bytes.misfire = if self.misfire { 1 } else { 0 };

		// Condition
		self.property_cmp.serialize_bytes(bytes.property_cmp).into_ok();

		// Arguments
		MaybeDigimonProperty::ref_cast(&self.arg_property)
			.serialize_bytes(bytes.arg_property)
			.into_ok();
		LittleEndian::write_u16(bytes.arg_num, self.arg_num);
		self.operation.serialize_bytes(bytes.operation).into_ok();

		// Zeros
		*bytes.zero_0 = 0;
		*bytes.zero_1 = [0; 0x5];
		*bytes.zero_2 = [0; 0xb];
		*bytes.zero_3 = [0; 0x4];
		*bytes.zero_4 = [0; 0x5];

		// And return OK
		Ok(())
	}
}

/// A possible effect condition
#[repr(transparent)]
#[derive(ref_cast::RefCast)]
#[derive(derive_more::From, derive_more::Into)]
pub struct MaybeEffectCondition(Option<EffectCondition>);

impl Bytes for MaybeEffectCondition {
	type ByteArray = [u8; 0x20];
	type DeserializeError = DeserializeBytesError;
	type SerializeError = <EffectCondition as Bytes>::SerializeError;

	fn deserialize_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::DeserializeError> {
		// If we have no property comparison, return None
		if bytes[0x2] == 0 {
			return Ok(Self(None));
		}

		// Else build the type
		Ok(Self(Some(EffectCondition::deserialize_bytes(bytes)?)))
	}

	#[allow(clippy::diverging_sub_expression)] // For if we ever change `EffectCondition::SerializeError`
	fn serialize_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::SerializeError> {
		// Check if we exist
		match self.0 {
			Some(cond) => cond.serialize_bytes(bytes)?,
			None => bytes[0x2] = 0,
		};

		// And return Ok
		Ok(())
	}
}
