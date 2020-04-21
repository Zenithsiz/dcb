// Crate
//--------------------------------------------------------------------------------------------------
	// Game
	use crate::game::{Bytes};
	use crate::game::card::property::{DigimonProperty, SupportConditionOperation};
//--------------------------------------------------------------------------------------------------

// byteorder
use byteorder::{ByteOrder, LittleEndian};

// Types
//--------------------------------------------------------------------------------------------------
	/// A digimon's support effect condition
	#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
	#[derive(serde::Serialize, serde::Deserialize)]
	pub struct SupportCondition
	{
		/// If the effect should throw a misfire if the condition isn't met
		misfire: bool,
		
		/// The condition type
		cond: DigimonProperty,
		
		/// The type argument
		type_arg: Option<DigimonProperty>,
		
		/// The number argument
		num_arg: u16,
		
		/// The operation
		operation: SupportConditionOperation,
		
		/// Unknown
		unknown: [u8; 16],
	}
	
	/// The error type thrown by `FromBytes`
	#[derive(Debug, derive_more::Display)]
	pub enum FromBytesError
	{
		/// Unable to read the condition
		#[display(fmt = "Unable to read the effect condition")]
		Condition( crate::game::card::property::digimon_property::FromBytesError ),
		
		/// Unable to read a property argument
		#[display(fmt = "Unable to read the property argument")]
		PropertyArgument( crate::game::card::property::digimon_property::FromBytesError ),
		
		/// Unable to read the effect operation
		#[display(fmt = "Unable to read the effect operation")]
		Operation( crate::game::card::property::support_condition_operation::FromBytesError ),
	}
	
	impl std::error::Error for FromBytesError {
		fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
			match self {
				Self::Condition(err) |
				Self::PropertyArgument(err) => Some(err),
				Self::Operation(err) => Some(err),
			}
		}
	}
//--------------------------------------------------------------------------------------------------

// Impl
//--------------------------------------------------------------------------------------------------
	impl SupportCondition
	{
		
	}
	
	// Bytes
	impl Bytes for SupportCondition
	{
		type ByteArray = [u8; 0x20];
		
		type FromError = FromBytesError;
		fn from_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::FromError>
		{
			// Get the condition
			let cond = DigimonProperty::from_bytes( &bytes[0x2] ).map_err(FromBytesError::Condition)?;
			
			// And return the move
			Ok( Self {
				misfire: { bytes[0x0] != 0 },
				cond,
				
				type_arg: if bytes[0x8] != 0 { Some(
					DigimonProperty::from_bytes( &bytes[0x8] ).map_err(FromBytesError::PropertyArgument)?
				)} else { None },
				
				num_arg: LittleEndian::read_u16( &bytes[0x14..0x16] ),
				
				operation: SupportConditionOperation::from_bytes( &bytes[0x1a] ).map_err(FromBytesError::Operation)?,
				
				unknown: [
					bytes[0x3], bytes[0x4], bytes[0x5], bytes[0x6], bytes[0x7],
					
					bytes[0x9], bytes[0xa ], bytes[0xb ], bytes[0xc ], bytes[0xd ], bytes[0xe], 
					bytes[0xf], bytes[0x10], bytes[0x11], bytes[0x12], bytes[0x13],
				]
			})
		}
		
		type ToError = !;
		fn to_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::ToError>
		{
			// 0x0 - Misfire
			bytes[0x0] = if self.misfire { 1 } else { 0 };
			
			// 0x1 - Always zero
			bytes[0x1] = 0;
			
			// 0x2 - Condition
			#[allow(clippy::diverging_sub_expression)] { // False positive
				self.cond.to_bytes(&mut bytes[0x2])
					.expect("Unable to convert condition to bytes");
			}
			
			// 0x3..0x8 - Unknown[0..5]
			bytes[0x3..0x8].copy_from_slice( &self.unknown[0..5] );
			
			// 0x8 - Type arg / 0 if None
			if let Some(type_arg) = self.type_arg {
				type_arg.to_bytes(&mut bytes[0x8])
					.expect("Unable to convert type argument to bytes")
			}
			else { bytes[0x8] = 0; }
			
			// 0x9..0x14 - Unknown[0x5..0x10]
			bytes[0x9..0x14].copy_from_slice( &self.unknown[0x5..0x10] );
			
			// 0x14..0x16 - Number arg
			LittleEndian::write_u16(&mut bytes[0x14..0x16], self.num_arg);
			
			// 0x1a - Operation arg
			#[allow(clippy::diverging_sub_expression)] { // False positive
				self.operation.to_bytes(&mut bytes[0x1a])
					.expect("Unable to convert operation to bytes");
			}
			
			// And return OK
			Ok(())
		}
	}
//--------------------------------------------------------------------------------------------------
