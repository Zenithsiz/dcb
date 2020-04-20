//! Digivolve

// Crate
//--------------------------------------------------------------------------------------------------
	// Game
	use crate::game::util;
	use crate::game::Bytes;
//--------------------------------------------------------------------------------------------------

// byteorder
//use byteorder::ByteOrder;
//use byteorder::LittleEndian;

// Macros
use serde::Serialize;
use serde::Deserialize;

// Types
//--------------------------------------------------------------------------------------------------
	/// A digivolve card
	#[derive(Debug, Serialize, Deserialize)]
	pub struct Digivolve
	{
		/// The basic info of the digivolve
		pub basic: Basic,
		
		/// The effects
		pub effects: Effects,
	}
	
	/// The basic properties of a digivolve
	#[derive(Debug, Serialize, Deserialize)]
	pub struct Basic
	{
		pub name: String,
		
		//unknown: u16,
	}
	
	#[derive(Debug, Serialize, Deserialize)]
	pub struct Effects
	{
		pub description: [String; 4],
		
		pub value0: u8,
		pub value1: u8,
		pub value2: u8,
		
		//arrow_color: Option<ArrowColor>,
		
		//conditions: SupportEffectConditions,
		//effects   : SupportEffects,
	}
	
	/*
	#[derive(Debug, Serialize, Deserialize)]
	struct SupportEffects
	{
		first : Option<SupportEffect>,
		second: Option<SupportEffect>,
		third : Option<SupportEffect>,
	}
	
	#[derive(Debug, Serialize, Deserialize)]
	struct SupportEffectConditions
	{
		first : Option<SupportEffectCondition>,
		second: Option<SupportEffectCondition>,
	}
	*/
	
	/// The error type thrown by `FromBytes`
	#[derive(Debug, derive_more::Display)]
	pub enum FromBytesError
	{
		/// Unable to convert name to a string
		#[display(fmt = "Unable to convert name to a string")]
		NameToString( util::ReadNullTerminatedStringError ),
		
		/// Unable to read the effect arrow color
		#[display(fmt = "Unable to read the effect arrow color")]
		EffectArrowColor( crate::game::card::property::arrow_color::UnknownArrowColor ),
		
		/// Unable to convert one of the support effect descriptions to a string
		#[display(fmt = "Unable to convert the {} support effect description to a string", rank)]
		SupportEffectDescriptionToString {
			rank: &'static str,
			err: util::ReadNullTerminatedStringError,
		},
		
		/// Unable to read a support effect condition
		#[display(fmt = "Unable to read the {0} support effect condition [digivolve:0x{1:x}]", rank, digivolve_pos)]
		SupportEffectCondition {
			rank: &'static str,
			digivolve_pos: u64,
			err: crate::game::card::property::support_condition::FromBytesError,
		},
		
		/// Unable to read a support effect
		#[display(fmt = "Unable to read the {} support effect", rank)]
		SupportEffect {
			rank: &'static str,
			err: crate::game::card::property::support_effect::FromBytesError,
		},
	}
	
	impl std::error::Error for FromBytesError {
		fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
			match self {
				Self::NameToString(err) |
				Self::SupportEffectDescriptionToString{err, ..} => Some(err),
				
				Self::EffectArrowColor(err) => Some(err),
				
				Self::SupportEffectCondition{err, ..} => Some(err),
				Self::SupportEffect{err, ..} => Some(err),
			}
		}
	}
	
	/// The error type thrown by `ToBytes`
	#[derive(Debug, derive_more::Display)]
	pub enum ToBytesError
	{
		/// The name was too big to be written to file
		#[display(fmt = "The name \"{}\" is too long to be written to file (max is 20)", _0)]
		NameTooLong( String ),
		
		/// The name was too big to be written to file
		#[display(fmt = "The {0} support effect description \"{1}\" is too long to be written to file (max is 21)", rank, string)]
		SupportEffectDescriptionTooLong {
			rank: &'static str,
			string: String,
		},
	}
	
	impl std::error::Error for ToBytesError {
		fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
			match self {
				Self::NameTooLong(..) |
				Self::SupportEffectDescriptionTooLong{ .. } => None,
			}
		}
	}
//--------------------------------------------------------------------------------------------------

// Impl
//--------------------------------------------------------------------------------------------------
	// Bytes
	impl Bytes for Digivolve
	{
		const BUF_BYTE_SIZE : usize = 0x6c;
		
		type FromError = FromBytesError;
		
		fn from_bytes(bytes: &[u8]) -> Result<Self, Self::FromError>
		{
			Ok( Self {
				basic: Basic {
					name: util::read_null_terminated_string( &bytes[0x0..0x15] ).map_err(FromBytesError::NameToString)?.to_string(),
				},
				
				effects: Effects {
					description: [
						util::read_null_terminated_string( &bytes[0x18..0x2d] ).map_err(|err| FromBytesError::SupportEffectDescriptionToString{ rank: "1st", err })?.to_string(),
						util::read_null_terminated_string( &bytes[0x2d..0x42] ).map_err(|err| FromBytesError::SupportEffectDescriptionToString{ rank: "2nd", err })?.to_string(),
						util::read_null_terminated_string( &bytes[0x42..0x57] ).map_err(|err| FromBytesError::SupportEffectDescriptionToString{ rank: "3rd", err })?.to_string(),
						util::read_null_terminated_string( &bytes[0x57..0x6c] ).map_err(|err| FromBytesError::SupportEffectDescriptionToString{ rank: "4th", err })?.to_string(),
					],
					
					value0: bytes[0x15],
					value1: bytes[0x16],
					value2: bytes[0x17],
				}
			})
		}
		
		type ToError = ToBytesError;
		
		fn to_bytes(&self, bytes: &mut [u8]) -> Result<(), Self::ToError>
		{
			// Basic
			//--------------------------------------------------------------------------------------------------
				// Write the name
				bytes[0x0..0x15].copy_from_slice( &{
					// Check if our name is too big
					if self.basic.name.len() >= 0x15 { return Err( ToBytesError::NameTooLong( self.basic.name.clone() ) ); }
					
					// Else make the buffer and copy everything over
					let mut buf = [0u8; 0x15];
					buf[ 0..self.basic.name.len() ].copy_from_slice( self.basic.name.as_bytes() );
					buf
				});
			//--------------------------------------------------------------------------------------------------
			
			// Effects
			//--------------------------------------------------------------------------------------------------
				// Write the support effects
				for (index, line) in self.effects.description.iter().enumerate()
				{
					bytes[0x18 + (0x15 * index) .. 0x2d + (0x15 * index)].copy_from_slice( &{
						// If the line is too big, return Err
						if line.len() >= 0x15 {
							return Err( ToBytesError::SupportEffectDescriptionTooLong {
								rank: match index {
									0 => "1st", 1 => "2nd",
									2 => "3rd", 3 => "4th",
									_ => unreachable!(),
								},
								
								string: line.clone()
							});
						}
						
						let mut buf = [0u8; 0x15];
						buf[ 0..line.len() ].copy_from_slice( line.as_bytes() );
						buf
					});
				}
				
				bytes[0x15] = self.effects.value0;
				bytes[0x16] = self.effects.value1;
				bytes[0x17] = self.effects.value2;
			//--------------------------------------------------------------------------------------------------
			
			// Return the bytes
			Ok(())
		}
	}
//--------------------------------------------------------------------------------------------------
