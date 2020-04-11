// Crate
//--------------------------------------------------------------------------------------------------
	// Game
	use crate::game::util;
	use crate::game::Bytes;
	use crate::game::FromBytes;
	use crate::game::ToBytes;
	use crate::game::card::property::Speciality;
	use crate::game::card::property::Level;
	use crate::game::card::property::Move;
	use crate::game::card::property::CrossMoveEffect;
	use crate::game::card::property::SupportEffectCondition;
	use crate::game::card::property::SupportEffect;
	use crate::game::card::property::ArrowColor;
//--------------------------------------------------------------------------------------------------

// byteorder
use byteorder::ByteOrder;
use byteorder::LittleEndian;

// Macros

use serde::Serialize;
use serde::Deserialize;
use contracts::*;

// Types
//--------------------------------------------------------------------------------------------------
	/// A digimon card
	#[derive(Debug, Serialize, Deserialize)]
	pub struct Digimon
	{
		/// The basic info of the digimon
		pub basic: Basic,
		
		/// The moves
		pub moves: Moves,
		
		/// The effects
		pub effects: Effects,
	}
	
	/// The basic properties of a digimon
	#[derive(Debug, Serialize, Deserialize)]
	pub struct Basic
	{
		pub name: String,
		pub speciality: Speciality,
		pub level: Level,
		pub dp_cost: u8,
		pub dp_give: u8,
		pub hp: u16,
		
		pub unknown: u16,
	}
	
	#[derive(Debug, Serialize, Deserialize)]
	pub struct Moves
	{
		pub circle: Move,
		pub triangle: Move,
		pub cross: Move,
	}
	
	#[derive(Debug, Serialize, Deserialize)]
	pub struct Effects
	{
		pub unknown: u8,
		
		pub cross_move: CrossMoveEffect,
		
		pub description: [String; 4],
		pub arrow_color: Option<ArrowColor>,
		
		pub conditions: SupportEffectConditions,
		pub effects   : SupportEffects,
	}
	
	#[derive(Debug, Serialize, Deserialize)]
	pub struct SupportEffects
	{
		pub first : Option<SupportEffect>,
		pub second: Option<SupportEffect>,
		pub third : Option<SupportEffect>,
	}
	
	#[derive(Debug, Serialize, Deserialize)]
	pub struct SupportEffectConditions
	{
		pub first : Option<SupportEffectCondition>,
		pub second: Option<SupportEffectCondition>,
	}
	
	/// The error type thrown by `FromBytes`
	#[derive(Debug, derive_more::Display)]
	pub enum FromBytesError
	{
		/// Unable to convert name to a string
		#[display(fmt = "Unable to convert name to a string")]
		NameToString( util::NullTerminatedStringError ),
		
		/// Unable to read the speciality
		#[display(fmt = "Unable to read the speciality")]
		Speciality( crate::game::card::property::speciality::UnknownSpeciality ),
		
		/// Unable to read the level
		#[display(fmt = "Unable to read the level")]
		Level( crate::game::card::property::level::UnknownLevel ),
		
		/// Unable to read the effect arrow color
		#[display(fmt = "Unable to read the effect arrow color")]
		EffectArrowColor( crate::game::card::property::arrow_color::UnknownArrowColor ),
		
		/// Unable to convert one of the support effect descriptions to a string
		#[display(fmt = "Unable to convert the {} support effect description to a string", rank)]
		SupportEffectDescriptionToString {
			rank: &'static str,
			
			
			err: util::NullTerminatedStringError,
		},
		
		/// Unable to read cross move effect
		#[display(fmt = "Unable to read the cross move effect")]
		CrossMoveEffect( crate::game::card::property::cross_move_effect::UnknownCrossMoveEffect ),
		
		/// Unable to read a support effect condition
		#[display(fmt = "Unable to read the {0} support effect condition [digimon:0x{1:x}]", rank, digimon_pos)]
		SupportEffectCondition {
			rank: &'static str,
			digimon_pos: u64,
			
			
			err: crate::game::card::property::support_effect_condition::FromBytesError,
		},
		
		/// Unable to read a support effect
		#[display(fmt = "Unable to read the {} support effect", rank)]
		SupportEffect {
			rank: &'static str,
			
			
			err: crate::game::card::property::support_effect::FromBytesError,
		},
		
		/// Unable to read a move
		#[display(fmt = "Unable to read the {} move", name)]
		Move {
			name: &'static str,
			
			
			err: crate::game::card::property::moves::FromBytesError,
		},
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
		
		/// Unable to write a move
		#[display(fmt = "Unable to write the {} move", name)]
		Move {
			name: &'static str,
			
			
			err: crate::game::card::property::moves::ToBytesError,
		},
	}
//--------------------------------------------------------------------------------------------------

// Impl
//--------------------------------------------------------------------------------------------------
	// Bytes
	impl Bytes for Digimon
	{
		const BUF_BYTE_SIZE : usize = 0x138;
	}
	
	// From bytes
	#[contract_trait]
	impl FromBytes for Digimon
	{
		type Output = Result<Self, FromBytesError>;
		
		fn from_bytes(bytes: &[u8]) -> Self::Output
		{
			// Assert some fields are 0
			//assert_eq!(bytes[0x1a], 0);
			
			// And return the struct
			Ok( Digimon {
				basic: Basic {
					name: util::read_null_terminated_string( &bytes[0x0..0x15] ).map_err(FromBytesError::NameToString)?.to_string(),
					
					speciality: Speciality::from_bytes( &[(bytes[0x17] & 0xF0) >> 4] ).map_err(FromBytesError::Speciality)?,
					level     :      Level::from_bytes( &[ bytes[0x17] & 0x0F      ] ).map_err(FromBytesError::Level     )?,
					
					dp_cost: bytes[0x18],
					dp_give: bytes[0x19],
					
					hp: LittleEndian::read_u16( &bytes[0x1b..0x1d] ),
					
					unknown: LittleEndian::read_u16( &bytes[0x15..0x17] ),
				},
				
				moves: Moves {
					circle  : Move::from_bytes( &bytes[0x1d..0x39] ).map_err(|err| FromBytesError::Move{ name: "circle"  , err })?,
					triangle: Move::from_bytes( &bytes[0x39..0x55] ).map_err(|err| FromBytesError::Move{ name: "triangle", err })?,
					cross   : Move::from_bytes( &bytes[0x55..0x71] ).map_err(|err| FromBytesError::Move{ name: "cross"   , err })?,
				},
				
				effects: Effects {
					cross_move: CrossMoveEffect::from_bytes( &[ bytes[0xe1] ] ).map_err(FromBytesError::CrossMoveEffect)?,
					
					description: [
						util::read_null_terminated_string( &bytes[0x0e4..0x0f9] ).map_err(|err| FromBytesError::SupportEffectDescriptionToString{ rank: "1st", err })?.to_string(),
						util::read_null_terminated_string( &bytes[0x0f9..0x10e] ).map_err(|err| FromBytesError::SupportEffectDescriptionToString{ rank: "2nd", err })?.to_string(),
						util::read_null_terminated_string( &bytes[0x10e..0x123] ).map_err(|err| FromBytesError::SupportEffectDescriptionToString{ rank: "3rd", err })?.to_string(),
						util::read_null_terminated_string( &bytes[0x123..0x138] ).map_err(|err| FromBytesError::SupportEffectDescriptionToString{ rank: "4th", err })?.to_string(),
					],
					
					arrow_color: if bytes[0xe3] != 0 {
						Some( ArrowColor::from_bytes( &bytes[0xe3..0xe4] ).map_err(FromBytesError::EffectArrowColor)? )
					} else { None },
					
					conditions: SupportEffectConditions {
						first: if bytes[0x73] != 0 { Some(
							SupportEffectCondition::from_bytes( &bytes[0x71..0x91] ).map_err(|err| FromBytesError::SupportEffectCondition{ rank: "1st", digimon_pos: 0x71, err })?
						)} else { None },
						
						second: if bytes[0x93] != 0 { Some(
							SupportEffectCondition::from_bytes( &bytes[0x91..0xb1] ).map_err(|err| FromBytesError::SupportEffectCondition{ rank: "2nd", digimon_pos: 0x91, err })?
						)} else { None },
					},
					
					effects: SupportEffects {
						first: if bytes[0xb1] != 0 { Some(
							SupportEffect::from_bytes( &bytes[0xb1..0xc1] ).map_err(|err| FromBytesError::SupportEffect{ rank: "1st", err })?
						)} else { None },
						
						second: if bytes[0xc1] != 0 { Some(
							SupportEffect::from_bytes( &bytes[0xc1..0xd1] ).map_err(|err| FromBytesError::SupportEffect{ rank: "2nd", err })?
						)} else { None },
						
						third: if bytes[0xd1] != 0 { Some(
							SupportEffect::from_bytes( &bytes[0xd1..0xe1] ).map_err(|err| FromBytesError::SupportEffect{ rank: "3rd", err })?
						)} else { None },
					},
					
					unknown: bytes[0xe2],
				},
			})
		}
	}
	
	// To bytes
	#[contract_trait]
	impl ToBytes for Digimon
	{
		type Output = Result<(), ToBytesError>;
		
		fn to_bytes(&self, bytes: &mut [u8]) -> Self::Output
		{
			// Assert the size is right
			assert_eq!(bytes.len(), Self::BUF_BYTE_SIZE);
			
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
				
				// Write the speciality and level bytes
				{
					let (mut speciality_byte, mut level_byte) = ( [0u8], [0u8] );
					
					self.basic.speciality.to_bytes(&mut speciality_byte);
					self.basic.level     .to_bytes(&mut      level_byte);
					
					// Merge them
					bytes[0x17] = (speciality_byte[0] << 4) | level_byte[0];
				}
				
				// DP / +P
				bytes[0x18] = self.basic.dp_cost;
				bytes[0x19] = self.basic.dp_give;
				
				// Health
				LittleEndian::write_u16(&mut bytes[0x1b..0x1d], self.basic.hp);
				
				LittleEndian::write_u16(&mut bytes[0x15..0x17], self.basic.unknown);
			//--------------------------------------------------------------------------------------------------
			
			// Moves
			self.moves.circle  .to_bytes(&mut bytes[0x1d..0x39]).map_err(|err| ToBytesError::Move{ name: "circle"  , err })?;
			self.moves.triangle.to_bytes(&mut bytes[0x39..0x55]).map_err(|err| ToBytesError::Move{ name: "triangle", err })?;
			self.moves.cross   .to_bytes(&mut bytes[0x55..0x71]).map_err(|err| ToBytesError::Move{ name: "cross"   , err })?;
			
			// Effects
			//--------------------------------------------------------------------------------------------------
				// Write the support effects
				for (index, line) in self.effects.description.iter().enumerate()
				{
					bytes[0x0e4 + (0x15 * index) .. 0x0f9 + (0x15 * index)].copy_from_slice( &{
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
				
				self.effects.cross_move.to_bytes(&mut bytes[0xe1..0xe2]);
				
				bytes[0xe2] = self.effects.unknown;
				
				if let Some(arrow_color) = self.effects.arrow_color { arrow_color.to_bytes( &mut bytes[0xe3..0xe4] ); }
				
				// If they are None, 0 is a valid value for the conditions
				if let Some(support_effect_condition) = &self.effects.conditions.first  { support_effect_condition.to_bytes(&mut bytes[0x71..0x91]); }
				if let Some(support_effect_condition) = &self.effects.conditions.second { support_effect_condition.to_bytes(&mut bytes[0x91..0xb1]); }
				
				
				// If they are None, 0 is a valid value for the effects
				if let Some(support_effect) = &self.effects.effects.first  { support_effect.to_bytes(&mut bytes[0xb1..0xc1]); }
				if let Some(support_effect) = &self.effects.effects.second { support_effect.to_bytes(&mut bytes[0xc1..0xd1]); }
				if let Some(support_effect) = &self.effects.effects.third  { support_effect.to_bytes(&mut bytes[0xd1..0xe1]); }
			//--------------------------------------------------------------------------------------------------
			
			// Return the bytes
			Ok(())
		}
	}
//--------------------------------------------------------------------------------------------------
