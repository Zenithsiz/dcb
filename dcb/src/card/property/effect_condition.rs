#![doc(include = "effect_condition.md")]

// Imports
use crate::card::property::{self, DigimonProperty, EffectConditionOperation, MaybeDigimonProperty};
use byteorder::{ByteOrder, LittleEndian};
use dcb_bytes::Bytes;
use dcb_util::{array_split, array_split_mut};
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

	/// Unknown field at `0x1`
	pub unknown_1: u8,

	/// Unknown field at `0x3`
	pub unknown_3: [u8; 0x5],

	/// Unknown field at `0x9`
	pub unknown_9: [u8; 0xb],

	/// Unknown field at `0x16`
	pub unknown_16: [u8; 0x4],

	/// Unknown field at `0x1b`
	pub unknown_1b: [u8; 0x5],
}

/// The error type thrown by `FromBytes`
#[derive(PartialEq, Eq, Clone, Copy, Debug, thiserror::Error)]
pub enum FromBytesError {
	/// Unable to read the condition
	#[error("Unable to read the effect condition")]
	Condition(#[source] property::digimon_property::FromBytesError),

	/// Unable to read a property argument
	#[error("Unable to read the property argument")]
	PropertyArgument(#[source] property::digimon_property::FromBytesError),

	/// Unable to read the effect operation
	#[error("Unable to read the effect operation")]
	Operation(#[source] property::effect_condition_operation::FromBytesError),
}

impl Bytes for EffectCondition {
	type ByteArray = [u8; 0x20];
	type FromError = FromBytesError;
	type ToError = !;

	fn from_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::FromError> {
		let bytes = array_split!(bytes,
			misfire     : 0x1,
			unknown_1   : 0x1,
			property_cmp: 0x1,
			unknown_3   : [0x5],
			arg_property: 0x1,
			unknown_9   : [0xb],
			arg_num     : [0x2],
			unknown_16  : [0x4],
			operation   : 1,
			unknown_1b  : [0x5],
		);

		Ok(Self {
			misfire:      (*bytes.misfire != 0),
			property_cmp: DigimonProperty::from_bytes(bytes.property_cmp).map_err(FromBytesError::Condition)?,

			arg_property: MaybeDigimonProperty::from_bytes(bytes.arg_property)
				.map_err(FromBytesError::PropertyArgument)?
				.into(),

			arg_num: LittleEndian::read_u16(bytes.arg_num),

			operation: EffectConditionOperation::from_bytes(bytes.operation).map_err(FromBytesError::Operation)?,

			unknown_1:  *bytes.unknown_1,
			unknown_3:  *bytes.unknown_3,
			unknown_9:  *bytes.unknown_9,
			unknown_16: *bytes.unknown_16,
			unknown_1b: *bytes.unknown_1b,
		})
	}

	fn to_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::ToError> {
		let bytes = array_split_mut!(bytes,
			misfire     : 0x1,
			unknown_1   : 0x1,
			property_cmp: 0x1,
			unknown_3   : [0x5],
			arg_property: 0x1,
			unknown_9   : [0xb],
			arg_num     : [0x2],
			unknown_16  : [0x4],
			operation   : 1,
			unknown_1b  : [0x5],
		);

		// Misfire
		*bytes.misfire = if self.misfire { 1 } else { 0 };

		// Condition
		self.property_cmp.to_bytes(bytes.property_cmp).into_ok();

		// Arguments
		MaybeDigimonProperty::ref_cast(&self.arg_property)
			.to_bytes(bytes.arg_property)
			.into_ok();
		LittleEndian::write_u16(bytes.arg_num, self.arg_num);
		self.operation.to_bytes(bytes.operation).into_ok();

		// Unknowns
		*bytes.unknown_1 = self.unknown_1;
		*bytes.unknown_3 = self.unknown_3;
		*bytes.unknown_9 = self.unknown_9;
		*bytes.unknown_16 = self.unknown_16;
		*bytes.unknown_1b = self.unknown_1b;

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
	type FromError = FromBytesError;
	type ToError = <EffectCondition as Bytes>::ToError;

	fn from_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::FromError> {
		// If we have no property comparison, return None
		if bytes[0x2] == 0 {
			return Ok(Self(None));
		}

		// Else build the type
		Ok(Self(Some(EffectCondition::from_bytes(bytes)?)))
	}

	#[allow(clippy::diverging_sub_expression)] // For if we ever change `EffectCondition::ToError`
	fn to_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::ToError> {
		// Check if we exist
		match self.0 {
			Some(cond) => cond.to_bytes(bytes)?,
			None => bytes[0x2] = 0,
		};

		// And return Ok
		Ok(())
	}
}
