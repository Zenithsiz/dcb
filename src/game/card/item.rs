//! Item

// Crate
//--------------------------------------------------------------------------------------------------
	// Game
	use crate::game::util;
	use crate::game::Bytes;
	use crate::game::card::property::EffectCondition;
	use crate::game::card::property::Effect;
	use crate::game::card::property::ArrowColor;
//--------------------------------------------------------------------------------------------------

// Array-ref
use arrayref::{array_ref, array_mut_ref};

// byteorder
use byteorder::ByteOrder;
use byteorder::LittleEndian;

// Macros
use serde::Serialize;
use serde::Deserialize;

// Types
//--------------------------------------------------------------------------------------------------
	/// A item card
	#[derive(Debug, Serialize, Deserialize)]
	pub struct Item
	{
		/// The basic info of the item
		pub basic: Basic,
		
		/// The effects
		effects: Effects,
	}
	
	/// The basic properties of a item
	#[derive(Debug, Serialize, Deserialize)]
	pub struct Basic
	{
		pub name: String,
		
		pub unknown: u16,
	}
	
	#[derive(Debug, Serialize, Deserialize)]
	struct Effects
	{
		description: [String; 4],
		arrow_color: Option<ArrowColor>,
		
		conditions: SupportConditions,
		effects   : SupportEffects,
	}
	
	#[derive(Debug, Serialize, Deserialize)]
	struct SupportEffects
	{
		first : Option<Effect>,
		second: Option<Effect>,
		third : Option<Effect>,
	}
	
	#[derive(Debug, Serialize, Deserialize)]
	struct SupportConditions
	{
		first : Option<EffectCondition>,
		second: Option<EffectCondition>,
	}
	
	/// The error type thrown by `FromBytes`
	#[derive(Debug, derive_more::Display)]
	pub enum FromBytesError
	{
		/// Unable to convert name to a string
		#[display(fmt = "Unable to convert name to a string")]
		NameToString( util::ReadNullTerminatedStringError ),
		
		/// Unable to read the effect arrow color
		#[display(fmt = "Unable to read the effect arrow color")]
		EffectArrowColor( crate::game::card::property::arrow_color::FromBytesError ),
		
		/// Unable to convert one of the support effect descriptions to a string
		#[display(fmt = "Unable to convert the {} support effect description to a string", rank)]
		SupportEffectDescriptionToString {
			rank: &'static str,
			err: util::ReadNullTerminatedStringError,
		},
		
		/// Unable to read a support effect condition
		#[display(fmt = "Unable to read the {0} support effect condition [item:0x{1:x}]", rank, item_pos)]
		SupportCondition {
			rank: &'static str,
			item_pos: u64,
			err: crate::game::card::property::effect_condition::FromBytesError,
		},
		
		/// Unable to read a support effect
		#[display(fmt = "Unable to read the {} support effect", rank)]
		SupportEffect {
			rank: &'static str,
			err: crate::game::card::property::effect::FromBytesError,
		},
	}
	
	impl std::error::Error for FromBytesError {
		fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
			match self {
				Self::NameToString(err) |
				Self::SupportEffectDescriptionToString{err, ..} => Some(err),
				
				Self::EffectArrowColor(err) => Some(err),
				Self::SupportCondition{err, ..} => Some(err),
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
	impl Bytes for Item
	{
		type ByteArray = [u8; 0xde];
		
		type FromError = FromBytesError;
		fn from_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::FromError>
		{
			// Assert some fields are 0
			//assert_eq!(bytes[0x1a], 0);
			
			// And return the struct
			Ok( Self {
				basic: Basic {
					name: util::read_null_terminated_string( &bytes[0x0..0x15] ).map_err(FromBytesError::NameToString)?.to_string(),
					
					unknown: LittleEndian::read_u16( &bytes[0x15..0x17] ),
				},
				
				effects: Effects {
					description: [
						util::read_null_terminated_string( &bytes[0x8a..0x9f] ).map_err(|err| FromBytesError::SupportEffectDescriptionToString{ rank: "1st", err })?.to_string(),
						util::read_null_terminated_string( &bytes[0x9f..0xb4] ).map_err(|err| FromBytesError::SupportEffectDescriptionToString{ rank: "2nd", err })?.to_string(),
						util::read_null_terminated_string( &bytes[0xb4..0xc9] ).map_err(|err| FromBytesError::SupportEffectDescriptionToString{ rank: "3rd", err })?.to_string(),
						util::read_null_terminated_string( &bytes[0xc9..0xde] ).map_err(|err| FromBytesError::SupportEffectDescriptionToString{ rank: "4th", err })?.to_string(),
					],
					
					arrow_color: if bytes[0x89] != 0 {
						Some( ArrowColor::from_bytes( &bytes[0x89] ).map_err(FromBytesError::EffectArrowColor)? )
					} else { None },
					
					conditions: SupportConditions {
						first: Option::<EffectCondition>::from_bytes( array_ref!(bytes, 0x19, 0x20) )
							.map_err(|err| FromBytesError::SupportCondition{ rank: "1st", item_pos: 0x19, err })?,
						second: Option::<EffectCondition>::from_bytes( array_ref!(bytes, 0x39, 0x20) )
							.map_err(|err| FromBytesError::SupportCondition{ rank: "2nd", item_pos: 0x39, err })?,
					},
					
					effects: SupportEffects {
						first: Option::<Effect>::from_bytes( array_ref!(bytes, 0x59, 0x10) )
							.map_err(|err| FromBytesError::SupportEffect{ rank: "1st", err })?,
						second: Option::<Effect>::from_bytes( array_ref!(bytes, 0x69, 0x10) )
							.map_err(|err| FromBytesError::SupportEffect{ rank: "2nd", err })?,
						third: Option::<Effect>::from_bytes( array_ref!(bytes, 0x79, 0x10) )
							.map_err(|err| FromBytesError::SupportEffect{ rank: "3rd", err })?,
					},
				},
			})
		}
		
		type ToError = ToBytesError;
		fn to_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::ToError>
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
				
				LittleEndian::write_u16(&mut bytes[0x15..0x17], self.basic.unknown);
			//--------------------------------------------------------------------------------------------------
			
			// Effects
			//--------------------------------------------------------------------------------------------------
				// Write the support effects
				for (index, line) in self.effects.description.iter().enumerate()
				{
					bytes[0x8a + (0x15 * index) .. 0x9f + (0x15 * index)].copy_from_slice( &{
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
				
				if let Some(arrow_color) = self.effects.arrow_color { arrow_color.to_bytes( &mut bytes[0x89] ).expect("Unable to convert arrow color to bytes"); }
				
				// If they are None, 0 is a valid value for the conditions
				self.effects.conditions.first .to_bytes( array_mut_ref!(bytes, 0x19, 0x20) )?;
				self.effects.conditions.second.to_bytes( array_mut_ref!(bytes, 0x39, 0x20) )?;
				
				
				// If they are None, 0 is a valid value for the effects
				self.effects.effects.first .to_bytes( array_mut_ref!(bytes, 0x59, 0x10) ).expect("TODO");
				self.effects.effects.second.to_bytes( array_mut_ref!(bytes, 0x69, 0x10) ).expect("TODO");
				self.effects.effects.third .to_bytes( array_mut_ref!(bytes, 0x79, 0x10) ).expect("TODO");
			//--------------------------------------------------------------------------------------------------
			
			// Return the bytes
			Ok(())
		}
	}
//--------------------------------------------------------------------------------------------------
