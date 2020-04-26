//! A digimon's effect condition
//! 
//! This module contains the [`EffectCondition`] struct, which describes a condition for an effect.
//! 
//! # Layout
//! Each support condition has a size of `0x20` bytes, and it's layout is the following:
//! 
//! | Offset | Size | Type                         | Name                      | Location       | Details                                                                            |
//! |--------|------|------------------------------|---------------------------|--------------- |------------------------------------------------------------------------------------|
//! | 0x0    | 0x0  | `bool`                       | Misfire                   | `misfire`      | If the condition throws a misfire when false                                       |
//! | 0x1    | 0x1  | `u8`                         |                           |                | Always zero                                                                        |
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
	Bytes,
	card::property::{
		self, DigimonProperty, EffectConditionOperation
	},
	//util,
};

/// A digimon's support effect condition
#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct EffectCondition
{
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
	unknown_3 : [u8; 0x5],
	unknown_9 : [u8; 0xb],
	unknown_16: [u8; 0x4],
	unknown_1b: [u8; 0x5],
}

/// The error type thrown by `FromBytes`
#[derive(Debug)]
#[derive(derive_more::Display, err_impl::Error)]
pub enum FromBytesError
{
	/// Unable to read the condition
	#[display(fmt = "Unable to read the effect condition")]
	Condition( #[error(source)] property::digimon_property::FromBytesError ),
	
	/// Unable to read a property argument
	#[display(fmt = "Unable to read the property argument")]
	PropertyArgument( #[error(source)] property::digimon_property::FromBytesError ),
	
	/// Unable to read the effect operation
	#[display(fmt = "Unable to read the effect operation")]
	Operation( #[error(source)] property::effect_condition_operation::FromBytesError ),
}

// Bytes
impl Bytes for Option<EffectCondition>
{
	type ByteArray = [u8; 0x20];
	
	type FromError = FromBytesError;
	fn from_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::FromError>
	{
		// If we have no property comparation, return None
		if bytes[0x2] == 0 {
			return Ok(None);
		}
		
		// Else build the type
		Ok( Some( EffectCondition {
			misfire: (bytes[0x0] != 0),
			property_cmp: DigimonProperty::from_bytes( &bytes[0x2] )
				.map_err(FromBytesError::Condition)?,
			
			arg_property: (bytes[0x8] != 0)
				.then(|| DigimonProperty::from_bytes( &bytes[0x8] ))
				.transpose()
				.map_err(FromBytesError::PropertyArgument)?,
			
			arg_num: LittleEndian::read_u16( &bytes[0x14..0x16] ),
			
			operation: EffectConditionOperation::from_bytes( &bytes[0x1a] )
				.map_err(FromBytesError::Operation)?,
			
			unknown_3: [ bytes[0x3], bytes[0x4], bytes[0x5], bytes[0x6], bytes[0x7] ],
			
			unknown_9: [
				bytes[0x9], bytes[0xa ], bytes[0xb ], bytes[0xc ], bytes[0xd ], bytes[0xe],
				bytes[0xf], bytes[0x10], bytes[0x11], bytes[0x12], bytes[0x13]
			],
			
			unknown_16: [ bytes[0x16], bytes[0x17], bytes[0x18], bytes[0x19] ],
			
			unknown_1b: [ bytes[0x1b], bytes[0x1c], bytes[0x1d], bytes[0x1e], bytes[0x1f] ],
		}))
	}
	
	type ToError = !;
	fn to_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::ToError>
	{
		// If we don't exist, write a `0` on the property comparation and return
		let cond = match self {
			Some(cond) => cond,
			None => {
				bytes[0x2] = 0;
				return Ok(());
			}
		};
		
		// 0x0 - Misfire
		bytes[0x0] = if cond.misfire { 1 } else { 0 };
		
		// 0x1 - Always zero
		bytes[0x1] = 0;
		
		// 0x2 - Condition
		cond.property_cmp.to_bytes(&mut bytes[0x2]).into_ok();
		
		// 0x3..0x8 - Unknown[0..5]
		//bytes[0x3..0x8].copy_from_slice( &self.unknown[0..5] );
		
		// 0x8 - Type arg / 0 if None
		if let Some(type_arg) = cond.arg_property {
			type_arg.to_bytes(&mut bytes[0x8]).into_ok();
		}
		else { bytes[0x8] = 0; }
		
		// 0x9..0x14 - Unknown[0x5..0x10]
		//bytes[0x9..0x14].copy_from_slice( &self.unknown[0x5..0x10] );
		
		// 0x14..0x16 - Number arg
		LittleEndian::write_u16(&mut bytes[0x14..0x16], cond.arg_num);
		
		// 0x1a - Operation arg
		cond.operation.to_bytes(&mut bytes[0x1a]).into_ok();
		
		// And return OK
		Ok(())
	}
}
