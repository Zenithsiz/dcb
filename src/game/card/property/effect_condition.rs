//! A digimon's effect condition
//!
//! This module contains the [`EffectCondition`] struct, which describes a condition for an effect.
//!
//! # Layout
//! Each support condition has a size of `0x20` bytes, and it's layout is the following:
//!
//! | Offset | Size | Type                         | Name                      | Location       | Details                                                                            |
//! |--------|------|------------------------------|---------------------------|--------------- |------------------------------------------------------------------------------------|
//! | 0x0    | 0x1  | `bool`                       | Misfire                   | `misfire`      | If the condition throws a misfire when false                                       |
//! | 0x1    | 0x1  | `u8`                         |                           | `unknown_1`    | Always zero                                                                        |
//! | 0x2    | 0x1  | [`DigimonProperty`]          | Property compare          | `property_cmp` | The property to compare to for the condition (or 0 if the condition doesn't exist) |
//! | 0x3    | 0x5  | `[u8; 0x5]`                  |                           | `unknown_3`    | Unknown                                                                            |
//! | 0x8    | 0x1  | `DigimonProperty`            | Property argument         | `arg_property` | Property argument for the comparation                                              |
//! | 0x9    | 0xb  | `[u8; 0xb]`                  |                           | `unknown_9`    | Unknown                                                                            |
//! | 0x14   | 0x2  | `u16`                        | Number argument           | `arg_num`      | Number argument for the comparation                                                |
//! | 0x16   | 0x4  | `[u8; 0x4]`                  |                           | `unknown_16`   | Unknown                                                                            |
//! | 0x1a   | 0x1  | [`EffectConditionOperation`] | Operation                 | `operation`    | Operation to use for the comparation                                               |
//! | 0x1b   | 0x5  | `[u8; 0x5]`                  |                           | `unknown_1b`   | Unknown                                                                            |

// byteorder
use byteorder::{ByteOrder, LittleEndian};

// Crate
use crate::game::{
	card::property::{self, DigimonProperty, EffectConditionOperation},
	util, Bytes,
};

/// A digimon's support effect condition
#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct EffectCondition {
	/// If the effect should throw a misfire when false
	misfire: bool,

	/// The property to compare to
	property_cmp: DigimonProperty,

	/// The property argument
	arg_property: Option<DigimonProperty>,

	/// The number argument
	arg_num: u16,

	/// The operation
	operation: EffectConditionOperation,

	// Unknown
	unknown_1: u8,
	unknown_3: [u8; 0x5],
	unknown_9: [u8; 0xb],
	unknown_16: [u8; 0x4],
	unknown_1b: [u8; 0x5],
}

/// The error type thrown by `FromBytes`
#[derive(Debug)]
#[derive(derive_more::Display, err_impl::Error)]
pub enum FromBytesError {
	/// Unable to read the condition
	#[display(fmt = "Unable to read the effect condition")]
	Condition(#[error(source)] property::digimon_property::FromBytesError),

	/// Unable to read a property argument
	#[display(fmt = "Unable to read the property argument")]
	PropertyArgument(#[error(source)] property::digimon_property::FromBytesError),

	/// Unable to read the effect operation
	#[display(fmt = "Unable to read the effect operation")]
	Operation(#[error(source)] property::effect_condition_operation::FromBytesError),
}

impl Bytes for EffectCondition {
	type ByteArray = [u8; 0x20];
	type FromError = FromBytesError;
	type ToError = !;

	fn from_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::FromError> {
		let bytes = util::array_split!(bytes,
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
			misfire: (*bytes.misfire != 0),
			property_cmp: DigimonProperty::from_bytes(bytes.property_cmp).map_err(FromBytesError::Condition)?,

			arg_property: Option::<DigimonProperty>::from_bytes(bytes.arg_property).map_err(FromBytesError::PropertyArgument)?,

			arg_num: LittleEndian::read_u16(bytes.arg_num),

			operation: EffectConditionOperation::from_bytes(bytes.operation).map_err(FromBytesError::Operation)?,

			unknown_1: *bytes.unknown_1,
			unknown_3: *bytes.unknown_3,
			unknown_9: *bytes.unknown_9,
			unknown_16: *bytes.unknown_16,
			unknown_1b: *bytes.unknown_1b,
		})
	}

	fn to_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::ToError> {
		let bytes = util::array_split_mut!(bytes,
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
		self.arg_property.to_bytes(bytes.arg_property).into_ok();
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

impl Bytes for Option<EffectCondition> {
	type ByteArray = [u8; 0x20];
	type FromError = FromBytesError;
	type ToError = <EffectCondition as crate::game::Bytes>::ToError;

	fn from_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::FromError> {
		// If we have no property comparation, return None
		if bytes[0x2] == 0 {
			return Ok(None);
		}

		// Else build the type
		Ok(Some(EffectCondition::from_bytes(bytes)?))
	}

	#[allow(clippy::diverging_sub_expression)] // For if we ever change `EffectCondition::ToError`
	fn to_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::ToError> {
		// Check if we exist
		match self {
			Some(cond) => cond.to_bytes(bytes)?,
			None => bytes[0x2] = 0,
		};

		// And return Ok
		Ok(())
	}
}
