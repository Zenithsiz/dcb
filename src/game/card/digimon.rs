//! A digimon card
//! 
//! This module stores the [Digimon] struct, which describes a digimon card.
//! 
//! # Layout
//! The digimon card has a size of 0x138 bytes, and it's layout is the following:
//! 
//! | Offset | Size | Type                 | Name                      | Location                       | Details                                                                             |
//! |--------|------|----------------------|---------------------------|--------------------------------|-------------------------------------------------------------------------------------|
//! | 0x0    | 0x15 | `char[0x15]`         | Name                      |` basic.name`                   |                                                                                     |
//! | 0x15   | 0x2  | `u16`                | Unknown                   |` basic.unknown_1`              | Most likely contains the digimon's model                                            |
//! | 0x17   | 0x1  | `u8`                 | Speciality & Level        |` basic.speciality basic.level` | The bottom nibble of this byte is the level, while the top nibble is the speciality |
//! | 0x18   | 0x1  | `u8`                 | DP                        |` basic.dp_cost`                |                                                                                     |
//! | 0x19   | 0x1  | `u8`                 | +P                        |` basic.dp_give`                |                                                                                     |
//! | 0x1a   | 0x1  | `u8`                 | Unknown                   |` basic.unknown_0`              | Is` 0` for all digimon                                                              |
//! | 0x1b   | 0x2  | `u16`                | Health                    |` basic.hp`                     |                                                                                     |
//! | 0x1d   | 0x1c | [`Move`]             | Circle Move               |` moves.circle`                 |                                                                                     |
//! | 0x39   | 0x1c | [`Move`]             | Triangle move             |` moves.triangle`               |                                                                                     |
//! | 0x55   | 0x1c | [`Move`]             | Cross move                |` moves.cross`                  |                                                                                     |
//! | 0x71   | 0x20 | [`SupportCondition`] | First condition           |` effects.conditions.first`     |                                                                                     |
//! | 0x91   | 0x20 | [`SupportCondition`] | Second condition          |` effects.conditions.second`    |                                                                                     |
//! | 0xb1   | 0x10 | [`SupportEffect`]    | First effect              |` support.effects.first`        |                                                                                     |
//! | 0xc1   | 0x10 | [`SupportEffect`]    | Second effect             |` support.effects.second`       |                                                                                     |
//! | 0xd1   | 0x10 | [`SupportEffect`]    | Third effect              |` support.effects.third`        |                                                                                     |
//! | 0xe1   | 0x1  | [`CrossMoveEffect`]  | Cross move effect         |` support.cross_move`           |                                                                                     |
//! | 0xe2   | 0x1  | `u8`                 | Unknown                   |` support.unknown`              |                                                                                     |
//! | 0xe3   | 0x1  | [`ArrowColor`]       | Effect arrow color        |` effects.arrow_color`          |                                                                                     |
//! | 0xe4   | 0x54 | `char[0x15][4]`      | Effect description lines  |` effects.description`          | Each line is` 0x15` bytes, split over 4 lines                                       |

// byteorder
use byteorder::{ByteOrder, LittleEndian};

// Crate
use crate::game::{
	util,
	Bytes,
	card::property::{
		Speciality, Level, Move, CrossMoveEffect, SupportCondition, SupportEffect, ArrowColor
	}
};

/// A digimon card
#[derive(PartialEq, Eq, Clone, Hash, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Digimon
{
	/// The basic info of the digimon
	pub basic: Basic,
	
	/// The moves
	pub moves: Moves,
	
	/// The support
	pub support: Support,
}

/// The basic properties of a digimon
#[derive(PartialEq, Eq, Clone, Hash, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Basic
{
	pub name: String,
	pub speciality: Speciality,
	pub level: Level,
	pub hp: u16,
	
	/// `DP` in the game.
	pub dp_cost: u8,
	
	/// `+P` in the game.
	pub dp_give: u8,
	
	// Unknown fields
	pub unknown_0: u8,
	pub unknown_1: u16,
}

/// The moves a digimon has
#[derive(PartialEq, Eq, Clone, Hash, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Moves
{
	pub circle  : Move,
	pub triangle: Move,
	pub cross   : Move,
}

/// The support effect of a digimon
#[derive(PartialEq, Eq, Clone, Hash, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Support
{
	/// Unknown field
	pub unknown: u8,
	
	/// The cross move effect
	pub cross_move: Option<CrossMoveEffect>,
	
	/// The effect description
	pub description: [String; 4],
	
	/// The effect arrow color
	pub arrow_color: Option<ArrowColor>,
	
	/// The effect conditions
	pub conditions: SupportConditions,
	
	/// The effects themselves
	pub effects: SupportSupport,
}

/// All of the support effects
#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct SupportSupport
{
	pub first : Option<SupportEffect>,
	pub second: Option<SupportEffect>,
	pub third : Option<SupportEffect>,
}

/// All of the support effect conditions
#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct SupportConditions
{
	pub first : Option<SupportCondition>,
	pub second: Option<SupportCondition>,
}

/// The error type thrown by [`FromBytes`]
#[derive(Debug, derive_more::Display)]
pub enum FromBytesError
{
	/// Unable to convert name to a string
	#[display(fmt = "Unable to convert name to a string")]
	NameToString( util::ReadNullTerminatedStringError ),
	
	/// Unable to convert one of the support effect descriptions to a string
	#[display(fmt = "The {} support effect description could not be converted to a string", rank)]
	SupportEffectDescriptionToString {
		rank: &'static str,
		
		
		err: util::ReadNullTerminatedStringError,
	},
	
	/// An unknown speciality was found
	#[display(fmt = "Unknown speciality found")]
	UnknownSpeciality( crate::game::card::property::speciality::UnknownSpeciality ),
	
	/// An unknown level was found
	#[display(fmt = "Unknown level found")]
	UnknownLevel( crate::game::card::property::level::UnknownLevel ),
	
	/// An unknown effect arrow color was found
	#[display(fmt = "Unknown effect arrow color found")]
	UnknownEffectArrowColor( crate::game::card::property::arrow_color::UnknownArrowColor ),
	
	/// An unknown cross move effect was found
	#[display(fmt = "Unknown cross move effect found")]
	UnknownCrossMoveEffect( crate::game::card::property::cross_move_effect::UnknownCrossMoveEffect ),
	
	/// Unable to read a support effect condition
	#[display(fmt = "Unable to read the {0} support effect condition", rank)]
	SupportCondition {
		rank: &'static str,
		
		
		err: crate::game::card::property::support_condition::FromBytesError,
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
	/// The name was too long to be written to file
	#[display(fmt = r#"The name "{}" is too long to be written to file"#, name)]
	NameTooLong {
		name: String,
		
		
		err: crate::game::util::WriteNullTerminatedStringError,
	},
	
	/// The name was not ascii
	#[display(fmt = r#"The name "{}" is not valid ascii"#, name)]
	NameNotAscii {
		name: String,
	},
	
	
	
	/// A support effect description was too long to be written to file
	#[display(fmt = r#"The {0} support effect description "{1}" is too long to be written to file"#, rank, string)]
	SupportEffectDescriptionTooLong {
		string: String,
		rank: String,
		
		
		err: crate::game::util::WriteNullTerminatedStringError,
	},
	
	/// A support effect description was not ascii
	#[display(fmt = r#"The {0} support effect description "{1}" is not valid ascii"#, rank, name)]
	SupportEffectDescriptionNotAscii {
		name: String,
		rank: String,
	},
	
	
	
	/// Unable to write a move
	#[display(fmt = "Unable to write the {} move", name)]
	Move {
		name: &'static str,
		
		
		err: crate::game::card::property::moves::ToBytesError,
	},
}

// Impl
//--------------------------------------------------------------------------------------------------
	// Bytes
	impl Bytes for Digimon
	{
		const BUF_BYTE_SIZE : usize = 0x138;
		
		type FromError = FromBytesError;
		
		fn from_bytes(bytes: &[u8]) -> Result<Self, Self::FromError>
		{
			// Note: We can't use `TryInto` because it only supports arrays up to 32
			// SAFETY: Safe as we checked the length
			assert!(bytes.len() == Self::BUF_BYTE_SIZE);
			let bytes: &[u8; Self::BUF_BYTE_SIZE] = unsafe {
				#[allow(clippy::as_conversions)]
				&*( bytes.as_ptr() as *const [u8; Self::BUF_BYTE_SIZE] )
			};
			
			// Return the struct after building it
			Ok( Self {
				// 0x0 - 0x1d
				basic: Basic {
					name      : util::read_null_terminated_string( &bytes[0x0..0x15] )  .map_err(FromBytesError::NameToString)?.to_string(),
					unknown_1 : LittleEndian::read_u16( &bytes[0x15..0x17] ),
					speciality: Speciality::from_bytes( &[(bytes[0x17] & 0xF0) >> 4] )  .map_err(FromBytesError::UnknownSpeciality)?,
					level     :      Level::from_bytes( &[(bytes[0x17] & 0x0F) >> 0] )  .map_err(FromBytesError::UnknownLevel     )?,
					dp_cost   : bytes[0x18],
					dp_give   : bytes[0x19],
					unknown_0 : bytes[0x1a],
					hp        : LittleEndian::read_u16( &bytes[0x1b..0x1d] ),
				},
				
				// 0x1d - 0x71
				moves: Moves {
					circle  : Move::from_bytes( &bytes[0x1d..0x39] )  .map_err(|err| FromBytesError::Move{ name: "circle"  , err })?,
					triangle: Move::from_bytes( &bytes[0x39..0x55] )  .map_err(|err| FromBytesError::Move{ name: "triangle", err })?,
					cross   : Move::from_bytes( &bytes[0x55..0x71] )  .map_err(|err| FromBytesError::Move{ name: "cross"   , err })?,
				},
				
				// 0x71 - 0x138
				support: Support {
					conditions: SupportConditions {
						first: if bytes[0x73] != 0 { Some(
							SupportCondition::from_bytes( &bytes[0x71..0x91] )  .map_err(|err| FromBytesError::SupportCondition{ rank: "1st", err })?
						)} else { None },
						
						second: if bytes[0x93] != 0 { Some(
							SupportCondition::from_bytes( &bytes[0x91..0xb1] )  .map_err(|err| FromBytesError::SupportCondition{ rank: "2nd", err })?
						)} else { None },
					},
					
					effects: SupportSupport {
						first: if bytes[0xb1] != 0 { Some(
							SupportEffect::from_bytes( &bytes[0xb1..0xc1] )  .map_err(|err| FromBytesError::SupportEffect{ rank: "1st", err })?
						)} else { None },
						
						second: if bytes[0xc1] != 0 { Some(
							SupportEffect::from_bytes( &bytes[0xc1..0xd1] )  .map_err(|err| FromBytesError::SupportEffect{ rank: "2nd", err })?
						)} else { None },
						
						third: if bytes[0xd1] != 0 { Some(
							SupportEffect::from_bytes( &bytes[0xd1..0xe1] )  .map_err(|err| FromBytesError::SupportEffect{ rank: "3rd", err })?
						)} else { None },
					},
					
					cross_move: if bytes[0xe1] != 0 { Some(
						CrossMoveEffect::from_bytes( &[ bytes[0xe1] ] )  .map_err(FromBytesError::UnknownCrossMoveEffect)?
					)} else { None },
					
					unknown: bytes[0xe2],
					
					arrow_color: if bytes[0xe3] != 0 {
						Some( ArrowColor::from_bytes( &bytes[0xe3..0xe4] )  .map_err(FromBytesError::UnknownEffectArrowColor)? )
					} else { None },
					
					description: [
						util::read_null_terminated_string( &bytes[0x0e4..0x0f9] )  .map_err(|err| FromBytesError::SupportEffectDescriptionToString{ rank: "1st", err })?.to_string(),
						util::read_null_terminated_string( &bytes[0x0f9..0x10e] )  .map_err(|err| FromBytesError::SupportEffectDescriptionToString{ rank: "2nd", err })?.to_string(),
						util::read_null_terminated_string( &bytes[0x10e..0x123] )  .map_err(|err| FromBytesError::SupportEffectDescriptionToString{ rank: "3rd", err })?.to_string(),
						util::read_null_terminated_string( &bytes[0x123..0x138] )  .map_err(|err| FromBytesError::SupportEffectDescriptionToString{ rank: "4th", err })?.to_string(),
					],
				},
			})
		}
		
		type ToError = ToBytesError;
		
		fn to_bytes(&self, bytes: &mut [u8]) -> Result<(), Self::ToError>
		{
			// Basic
			//--------------------------------------------------------------------------------------------------
				// Name
				// If it's not valid ascii, return Err
				// If we cannot write it to the buffer, return Err
				if !self.basic.name.chars().all(|c| c.is_ascii() && !c.is_ascii_control()) { return Err( ToBytesError::NameNotAscii{name: self.basic.name.clone()} ); }
				bytes[0x0..0x15].copy_from_slice(
					util::write_null_terminated_string(&self.basic.name, &mut [0u8; 0x15])
						.map_err(|err| ToBytesError::NameTooLong{ name: self.basic.name.clone(), err })?
				);
				
				// Unknown 1
				LittleEndian::write_u16(&mut bytes[0x15..0x17], self.basic.unknown_1);
				
				// Speciality / Level
				{
					let (mut speciality_byte, mut level_byte) = ( [0u8], [0u8] );
					
					self.basic.speciality.to_bytes(&mut speciality_byte)?;
					self.basic.level     .to_bytes(&mut      level_byte)?;
					
					// Merge them
					bytes[0x17] = (speciality_byte[0] << 4) | level_byte[0];
				}
				
				// DP / +P
				bytes[0x18] = self.basic.dp_cost;
				bytes[0x19] = self.basic.dp_give;
				
				// Unknown
				bytes[0x1a] = self.basic.unknown_0;
				
				// Health
				LittleEndian::write_u16(&mut bytes[0x1b..0x1d], self.basic.hp);
			//--------------------------------------------------------------------------------------------------
			
			// Moves
			self.moves.circle  .to_bytes(&mut bytes[0x1d..0x39]).map_err(|err| ToBytesError::Move{ name: "circle"  , err })?;
			self.moves.triangle.to_bytes(&mut bytes[0x39..0x55]).map_err(|err| ToBytesError::Move{ name: "triangle", err })?;
			self.moves.cross   .to_bytes(&mut bytes[0x55..0x71]).map_err(|err| ToBytesError::Move{ name: "cross"   , err })?;
			
			// Support
			// Note: Although support conditions and effects aren't written if they're None,
			//       a bit pattern of all 0s is a valid pattern and means "None" to the game.
			//--------------------------------------------------------------------------------------------------
				// Support conditions
				if let Some(support_condition) = &self.support.conditions.first  { support_condition.to_bytes(&mut bytes[0x71..0x91])?; }
				if let Some(support_condition) = &self.support.conditions.second { support_condition.to_bytes(&mut bytes[0x91..0xb1])?; }
				
				// Support effects
				if let Some(support_effect) = &self.support.effects.first  { support_effect.to_bytes(&mut bytes[0xb1..0xc1])?; }
				if let Some(support_effect) = &self.support.effects.second { support_effect.to_bytes(&mut bytes[0xc1..0xd1])?; }
				if let Some(support_effect) = &self.support.effects.third  { support_effect.to_bytes(&mut bytes[0xd1..0xe1])?; }
				
				// Cross move
				if let Some(cross_move) = self.support.cross_move { cross_move.to_bytes(&mut bytes[0xe1..0xe2])? };
				
				// Unknown
				bytes[0xe2] = self.support.unknown;
				
				// Support arrow color
				if let Some(arrow_color) = self.support.arrow_color { arrow_color.to_bytes( &mut bytes[0xe3..0xe4] )?; }
				
				// Write the support effects
				for (index, line) in self.support.description.iter().enumerate()
				{
					// If it's not valid ascii, return Err
					if !line.chars().all(|c| c.is_ascii() && !c.is_ascii_control()) {
						return Err( ToBytesError::SupportEffectDescriptionNotAscii{
							name: line.clone(),
							rank: util::as_ordinal((index+1) as u64),
						});
					}
					
					// If we cannot write it to the buffer, return Err
					bytes[0x0e4 + (0x15 * index) .. 0x0f9 + (0x15 * index)].copy_from_slice(
						util::write_null_terminated_string(line, &mut [0u8; 0x15])
							.map_err(|err| ToBytesError::SupportEffectDescriptionTooLong {
								string: line.clone(),
								rank: util::as_ordinal((index+1) as u64),
								err
							})?
					);
				}
			//--------------------------------------------------------------------------------------------------
			
			// Return Ok
			Ok(())
		}
	}
//--------------------------------------------------------------------------------------------------
