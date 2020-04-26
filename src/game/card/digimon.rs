//! A digimon card
//! 
//! This module stores the [`Digimon`] struct, which describes a digimon card.
//! 
//! # Layout
//! The digimon card has a size of `0x138` bytes, and it's layout is the following:
//! 
//! | Offset | Size | Type                 | Name                      | Location               | Details                                                                             |
//! |--------|------|----------------------|---------------------------|------------------------|-------------------------------------------------------------------------------------|
//! | 0x0    | 0x15 | `[char; 0x15]`       | Name                      | `name`                 | Null-terminated                                                                     |
//! | 0x15   | 0x2  | `u16`                | Unknown                   | `unknown_15`           | Most likely contains the digimon's model                                            |
//! | 0x17   | 0x1  | `u8`                 | Speciality & Level        | `speciality level`     | The bottom nibble of this byte is the level, while the top nibble is the speciality |
//! | 0x18   | 0x1  | `u8`                 | DP                        | `dp_cost`              |                                                                                     |
//! | 0x19   | 0x1  | `u8`                 | +P                        | `dp_give`              |                                                                                     |
//! | 0x1a   | 0x1  | `u8`                 | Unknown                   | `unknown_1a`           | Is `0` for all digimon                                                              |
//! | 0x1b   | 0x2  | `u16`                | Health                    | `hp`                   |                                                                                     |
//! | 0x1d   | 0x1c | [`Move`]             | Circle Move               | `move_circle`          |                                                                                     |
//! | 0x39   | 0x1c | [`Move`]             | Triangle move             | `move_triangle`        |                                                                                     |
//! | 0x55   | 0x1c | [`Move`]             | Cross move                | `move_cross`           |                                                                                     |
//! | 0x71   | 0x20 | [`SupportCondition`] | First condition           | `effect_conditions[0]` |                                                                                     |
//! | 0x91   | 0x20 | [`SupportCondition`] | Second condition          | `effect_conditions[1]` |                                                                                     |
//! | 0xb1   | 0x10 | [`SupportEffect`]    | First effect              | `effects[0]`           |                                                                                     |
//! | 0xc1   | 0x10 | [`SupportEffect`]    | Second effect             | `effects[1]`           |                                                                                     |
//! | 0xd1   | 0x10 | [`SupportEffect`]    | Third effect              | `effects[2]`           |                                                                                     |
//! | 0xe1   | 0x1  | [`CrossMoveEffect`]  | Cross move effect         | `cross_move_effect`    |                                                                                     |
//! | 0xe2   | 0x1  | `u8`                 | Unknown                   | `unknown_e2`           |                                                                                     |
//! | 0xe3   | 0x1  | [`ArrowColor`]       | Effect arrow color        | `effect_arrow_color`   |                                                                                     |
//! | 0xe4   | 0x54 | `[[char; 0x15]; 4]`  | Effect description lines  | `effect_description`   | Each line is` 0x15` bytes, split over 4 lines, each null terminated                 |

// byteorder
use byteorder::{ByteOrder, LittleEndian};

// Crate
use crate::game::{
	util,
	Bytes,
	card::property::{
		self,
		Speciality,
		Level,
		Move,
		CrossMoveEffect,
		SupportCondition,
		SupportEffect,
		ArrowColor,
	}
};

/// A digimon card
/// 
/// Contains all information about each digimon card stored in the [`Card Table`](crate::game::card::table::Table)
#[derive(PartialEq, Eq, Clone, Hash, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Digimon
{
	/// The digimon's name
	/// 
	/// An ascii string with 20 characters at most
	pub name: ascii::AsciiString,
	
	/// The digimon's speciality
	/// 
	/// Stored alongside with the level in a single byte
	pub speciality: Speciality,
	
	/// The digimon's level
	/// 
	/// Stored alongside with the speciality in a single byte
	pub level: Level,
	
	/// The digimon's health points
	pub hp: u16,
	
	/// The DP cost to play this digimon card
	/// 
	/// `DP` in the game.
	pub dp_cost: u8,
	
	/// The number of DP given when discarded
	/// 
	/// `+P` in the game.
	pub dp_give: u8,
	
	// Unknown fields
	pub unknown_1a: u8,
	pub unknown_15: u16,
	pub unknown_e2: u8,
	
	/// The digimon's circle move
	pub move_circle: Move,
	
	/// The digimon's triangle move
	pub move_triangle: Move,
	
	/// The digimon's cross move
	pub move_cross: Move,
	
	/// The digimon's cross move effect, if any
	#[serde(default)]
	pub cross_move_effect: Option<CrossMoveEffect>,
	
	/// The digimon's effect description.
	/// 
	/// The description is split along 4 lines, each
	/// being an ascii string with 20 characters at most.
	pub effect_description: [ascii::AsciiString; 4],
	
	/// The effect arrow color
	#[serde(default)]
	pub effect_arrow_color: Option<ArrowColor>,
	
	/// The effect conditions
	#[serde(default)]
	pub effect_conditions: [Option<SupportCondition>; 2],
	
	/// The effects themselves
	#[serde(default)]
	pub effects: [Option<SupportEffect>; 3],
}

/// Error type for [`Bytes::from_bytes`]
#[derive(Debug)]
#[derive(derive_more::Display, err_impl::Error)]
pub enum FromBytesError
{
	/// Unable to read the digimon name
	#[display(fmt = "Unable to read the digimon name")]
	Name( #[error(source)] util::ReadNullAsciiStringError ),
	
	/// Unable to read the first support effect description
	#[display(fmt = "Unable to read the first line of the effect description")]
	EffectDescriptionFirst( #[error(source)] util::ReadNullAsciiStringError ),
	
	/// Unable to read the second support effect description
	#[display(fmt = "Unable to read the second line of the effect description")]
	EffectDescriptionSecond( #[error(source)] util::ReadNullAsciiStringError ),
	
	/// Unable to read the third support effect description
	#[display(fmt = "Unable to read the third line of the effect description")]
	EffectDescriptionThird( #[error(source)] util::ReadNullAsciiStringError ),
	
	/// Unable to read the fourth support effect description
	#[display(fmt = "Unable to read the fourth line of the effect description")]
	EffectDescriptionFourth( #[error(source)] util::ReadNullAsciiStringError ),
	
	/// An unknown speciality was found
	#[display(fmt = "Unknown speciality found")]
	Speciality( #[error(source)] property::speciality::FromBytesError ),
	
	/// An unknown level was found
	#[display(fmt = "Unknown level found")]
	Level( #[error(source)] property::level::FromBytesError ),
	
	/// An unknown effect arrow color was found
	#[display(fmt = "Unknown effect arrow color found")]
	ArrowColor( #[error(source)] property::arrow_color::FromBytesError ),
	
	/// An unknown cross move effect was found
	#[display(fmt = "Unknown cross move effect found")]
	CrossMoveEffect( #[error(source)] property::cross_move_effect::FromBytesError ),
	
	/// Unable to read the circle move
	#[display(fmt = "Unable to read the circle move")]
	MoveCircle( #[error(source)] property::moves::FromBytesError ),
	
	/// Unable to read the triangle move
	#[display(fmt = "Unable to read the triangle move")]
	MoveTriangle( #[error(source)] property::moves::FromBytesError ),
	
	/// Unable to read the cross move
	#[display(fmt = "Unable to read the cross move")]
	MoveCross( #[error(source)] property::moves::FromBytesError ),
	
	/// Unable to read the first effect condition
	#[display(fmt = "Unable to read the first effect condition")]
	EffectConditionFirst( #[error(source)] property::support_condition::FromBytesError ),
	
	/// Unable to read the second effect condition
	#[display(fmt = "Unable to read the second effect condition")]
	EffectConditionSecond( #[error(source)] property::support_condition::FromBytesError ),
	
	/// Unable to read the first effect
	#[display(fmt = "Unable to read the first effect")]
	EffectFirst( #[error(source)] property::support_effect::FromBytesError ),
	
	/// Unable to read the second effect
	#[display(fmt = "Unable to read the second effect")]
	EffectSecond( #[error(source)] property::support_effect::FromBytesError ),
	
	/// Unable to read the third effect
	#[display(fmt = "Unable to read the third effect")]
	EffectThird( #[error(source)] property::support_effect::FromBytesError ),
}
/// Error type for [`Bytes::to_bytes`]
#[derive(Debug)]
#[derive(derive_more::Display, err_impl::Error)]
pub enum ToBytesError
{
	/// Unable to write the digimon name
	#[display(fmt = "Unable to write the digimon name")]
	Name( #[error(source)] util::WriteNullAsciiStringError ),
	
	/// Unable to write the first support effect description
	#[display(fmt = "Unable to write the first line of the effect description")]
	EffectDescriptionFirst( #[error(source)] util::WriteNullAsciiStringError ),
	
	/// Unable to write the second support effect description
	#[display(fmt = "Unable to write the second line of the effect description")]
	EffectDescriptionSecond( #[error(source)] util::WriteNullAsciiStringError ),
	
	/// Unable to write the third support effect description
	#[display(fmt = "Unable to write the third line of the effect description")]
	EffectDescriptionThird( #[error(source)] util::WriteNullAsciiStringError ),
	
	/// Unable to write the fourth support effect description
	#[display(fmt = "Unable to write the fourth line of the effect description")]
	EffectDescriptionFourth( #[error(source)] util::WriteNullAsciiStringError ),
	
	/// Unable to write the circle move
	#[display(fmt = "Unable to write the circle move")]
	MoveCircle( #[error(source)] property::moves::ToBytesError ),
	
	/// Unable to write the triangle move
	#[display(fmt = "Unable to write the triangle move")]
	MoveTriangle( #[error(source)] property::moves::ToBytesError ),
	
	/// Unable to write the cross move
	#[display(fmt = "Unable to write the cross move")]
	MoveCross( #[error(source)] property::moves::ToBytesError ),
}

impl Bytes for Digimon
{
	type ByteArray = [u8; 0x138];
	
	type FromError = FromBytesError;
	fn from_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::FromError>
	{
		// Get all byte arrays we need
		let bytes = util::array_split!(bytes,
			0x00..0x15  => name,
			0x15..0x17  => unknown_15,
			=0x17       => speciality_level,
			=0x18       => dp_cost,
			=0x19       => dp_give,
			=0x1a       => unknown_1a,
			0x1b..0x1d  => hp,
			0x1d..0x39  => move_circle,
			0x39..0x55  => move_triangle,
			0x55..0x71  => move_cross,
			0x71..0x91  => condition_first,
			0x91..0xb1  => condition_second,
			0xb1..0xc1  => effect_first,
			0xc1..0xd1  => effect_second,
			0xd1..0xe1  => effect_third,
			0xe1..0xe2  => cross_move_effect,
			0xe2..0xe3  => unknown_e2,
			0xe3..0xe4  => effect_arrow_color,
			0xe4..0x138 => effect_description,
		);
		
		// Return the struct after building it
		Ok( Self {
			// 0x0 - 0x1d
			name: util::read_null_ascii_string(bytes.name)
				.map_err(FromBytesError::Name)?
				.chars().collect(),
			
			unknown_15: LittleEndian::read_u16(bytes.unknown_15),
			
			speciality: Speciality::from_bytes( &( (bytes.speciality_level & 0xF0) >> 4 ) )
				.map_err(FromBytesError::Speciality)?,
			
			level: Level::from_bytes( &( (bytes.speciality_level & 0x0F) >> 0 ) )
				.map_err(FromBytesError::Level)?,
			
			dp_cost   : *bytes.dp_cost,
			dp_give   : *bytes.dp_give,
			unknown_1a: *bytes.unknown_1a,
			
			hp: LittleEndian::read_u16( bytes.hp ),
			
			// 0x1d - 0x71
			move_circle: Move::from_bytes( bytes.move_circle )
				.map_err(FromBytesError::MoveCircle)?,
			move_triangle: Move::from_bytes( bytes.move_triangle )
				.map_err(FromBytesError::MoveTriangle)?,
			move_cross: Move::from_bytes( bytes.move_cross )
				.map_err(FromBytesError::MoveCross)?,
			
			// 0x71 - 0x138
			effect_conditions: [
				(bytes.condition_first[2] != 0)
					.then(|| SupportCondition::from_bytes( bytes.condition_first ) )
					.transpose()
					.map_err(FromBytesError::EffectConditionFirst)?,
				
				(bytes.condition_second[2] != 0)
					.then(|| SupportCondition::from_bytes( bytes.condition_second ) )
					.transpose()
					.map_err(FromBytesError::EffectConditionSecond)?,
			],
			
			effects: [
				(bytes.effect_first[0] != 0)
					.then(|| SupportEffect::from_bytes( bytes.effect_first ) )
					.transpose()
					.map_err(FromBytesError::EffectFirst)?,
				
				(bytes.effect_second[0] != 0)
					.then(|| SupportEffect::from_bytes( bytes.effect_second ) )
					.transpose()
					.map_err(FromBytesError::EffectSecond)?,
				
				(bytes.effect_third[0] != 0)
					.then(|| SupportEffect::from_bytes( bytes.effect_third ) )
					.transpose()
					.map_err(FromBytesError::EffectThird)?,
			],
			
			cross_move_effect: (bytes.cross_move_effect[0] != 0)
				.then(|| CrossMoveEffect::from_bytes( &bytes.cross_move_effect[0] ) )
				.transpose()
				.map_err(FromBytesError::CrossMoveEffect)?,
			
			unknown_e2: bytes.unknown_e2[0],
			
			effect_arrow_color: (bytes.effect_arrow_color[0] != 0)
				.then(|| ArrowColor::from_bytes( &bytes.effect_arrow_color[0] ) )
				.transpose()
				.map_err(FromBytesError::ArrowColor)?,
			
			effect_description: [
				util::read_null_ascii_string( &bytes.effect_description[0x00..0x15] )
					.map_err(FromBytesError::EffectDescriptionFirst)?
					.chars().collect(),
				util::read_null_ascii_string( &bytes.effect_description[0x15..0x2a] )
					.map_err(FromBytesError::EffectDescriptionSecond)?
					.chars().collect(),
				util::read_null_ascii_string( &bytes.effect_description[0x2a..0x3f] )
					.map_err(FromBytesError::EffectDescriptionThird)?
					.chars().collect(),
				util::read_null_ascii_string( &bytes.effect_description[0x3f..0x54] )
					.map_err(FromBytesError::EffectDescriptionFourth)?
					.chars().collect(),
			],
		})
	}
	
	type ToError = ToBytesError;
	fn to_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::ToError>
	{
		// Get all byte arrays we need
		let bytes = util::array_split_mut!(bytes,
			name                : [0x15],
			unknown_15          : [0x2],
			speciality_level    : 0x1,
			dp_cost             : 0x1,
			dp_give             : 0x1,
			unknown_1a          : 0x1,
			hp                  : [0x2],
			move_circle         : [0x1c],
			move_triangle       : [0x1c],
			move_cross          : [0x1c],
			condition_first     : [0x20],
			condition_second    : [0x20],
			effect_first        : [0x10],
			effect_second       : [0x10],
			effect_third        : [0x10],
			cross_move_effect   : 1,
			unknown_e2          : 1,
			effect_arrow_color  : 1,
			effect_description_0: [0x15],
			effect_description_1: [0x15],
			effect_description_2: [0x15],
			effect_description_3: [0x15],
		);
		
		// name
		util::write_null_ascii_string(self.name.as_ref(), bytes.name)
			.map_err(ToBytesError::Name)?;
		
		// unknown_15
		LittleEndian::write_u16(bytes.unknown_15, self.unknown_15);
		
		// Speciality / Level
		{
			let (mut speciality_byte, mut level_byte) = ( 0u8, 0u8 );
			
			// Note: Buffers have 1 byte, so this can't fail
			self.speciality.to_bytes(&mut speciality_byte)?;
			self.level.to_bytes(&mut level_byte)?;
			
			// Merge them
			*bytes.speciality_level = (speciality_byte << 4) | level_byte;
		}
		
		// DP / +P
		*bytes.dp_cost = self.dp_cost;
		*bytes.dp_give = self.dp_give;
		
		// Unknown
		*bytes.unknown_1a = self.unknown_1a;
		
		// Health
		LittleEndian::write_u16(bytes.hp, self.hp);
		
		// Moves
		self.  move_circle.to_bytes( bytes.move_circle   ).map_err(ToBytesError::MoveCircle  )?;
		self.move_triangle.to_bytes( bytes.move_triangle ).map_err(ToBytesError::MoveTriangle)?;
		self.   move_cross.to_bytes( bytes.move_cross    ).map_err(ToBytesError::MoveCross   )?;
	
		// Support conditions
		// Note: Although support conditions and effects aren't written if they're None,
		//       a bit pattern of all 0s is a valid pattern and means "None" to the game.
		if let Some(support_condition) = &self.effect_conditions[0] { support_condition.to_bytes( bytes.condition_first  )?; }
		if let Some(support_condition) = &self.effect_conditions[1] { support_condition.to_bytes( bytes.condition_second )?; }
		
		// Support effects
		if let Some(support_effect) = &self.effects[0] { support_effect.to_bytes( bytes.effect_first  )?; }
		if let Some(support_effect) = &self.effects[1] { support_effect.to_bytes( bytes.effect_second )?; }
		if let Some(support_effect) = &self.effects[2] { support_effect.to_bytes( bytes.effect_third  )?; }
		
		// Cross move
		if let Some(move_cross) = self.cross_move_effect { move_cross.to_bytes( bytes.cross_move_effect )? };
		
		// Unknown
		*bytes.unknown_e2 = self.unknown_e2;
		
		// Support arrow color
		if let Some(arrow_color) = self.effect_arrow_color { arrow_color.to_bytes( bytes.effect_arrow_color )? }
		
		// effect_description
		util::write_null_ascii_string(self.effect_description[0].as_ref(), bytes.effect_description_0)
			.map_err(ToBytesError::EffectDescriptionFirst)?;
		util::write_null_ascii_string(self.effect_description[1].as_ref(), bytes.effect_description_1)
			.map_err(ToBytesError::EffectDescriptionSecond)?;
		util::write_null_ascii_string(self.effect_description[2].as_ref(), bytes.effect_description_2)
			.map_err(ToBytesError::EffectDescriptionThird)?;
		util::write_null_ascii_string(self.effect_description[3].as_ref(), bytes.effect_description_3)
			.map_err(ToBytesError::EffectDescriptionFourth)?;
		
		// Return Ok
		Ok(())
	}
}
