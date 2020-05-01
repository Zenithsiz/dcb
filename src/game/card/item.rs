//! An item card
//! 
//! This module contains the [`Item`] struct, which describes an item card.
//! 
//! # Layout
//! The item card has a size of `0xde` bytes, and it's layout is the following:
//! 
//! | Offset | Size | Type                | Name                      | Location               | Details                                                                             |
//! |--------|------|---------------------|---------------------------|------------------------|-------------------------------------------------------------------------------------|
//! | 0x0    | 0x15 | `[char; 0x15]`      | Name                      | `name`                 | Null-terminated                                                                     |
//! | 0x15   | 0x4  | `u32`               | Unknown                   | `unknown_15`           |                                                                                     |
//! | 0x19   | 0x20 | [`EffectCondition`] | First condition           | `effect_conditions[0]` |                                                                                     |
//! | 0x39   | 0x20 | [`EffectCondition`] | Second condition          | `effect_conditions[1]` |                                                                                     |
//! | 0x59   | 0x10 | [`Effect`]          | First effect              | `effects[0]`           |                                                                                     |
//! | 0x69   | 0x10 | [`Effect`]          | Second effect             | `effects[1]`           |                                                                                     |
//! | 0x79   | 0x10 | [`Effect`]          | Third effect              | `effects[2]`           |                                                                                     |
//! | 0x89   | 0x1  | [`ArrowColor`]      | Effect arrow color        | `effect_arrow_color`   |                                                                                     |
//! | 0x8a   | 0x54 | `[[char; 0x15]; 4]` | Effect description lines  | `effect_description`   | Each line is` 0x15` bytes, split over 4 lines, each null terminated                 |

// byteorder
use byteorder::{ByteOrder, LittleEndian};

// Crate
use crate::game::{
	util,
	Bytes,
	card::property::{
		self,
		EffectCondition,
		Effect,
		ArrowColor,
	}
};

/// A item card
#[derive(PartialEq, Eq, Clone, Hash, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Item
{
	/// The digimon's name
	/// 
	/// An ascii string with 20 characters at most
	pub name: ascii::AsciiString,
	
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
	pub effect_conditions: [Option<EffectCondition>; 2],
	
	/// The effects themselves
	#[serde(default)]
	pub effects: [Option<Effect>; 3],
	
	// Unknown fields
	pub unknown_15: u32,
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
	
	/// An unknown effect arrow color was found
	#[display(fmt = "Unknown effect arrow color found")]
	ArrowColor( #[error(source)] property::arrow_color::FromBytesError ),
	
	/// Unable to read the first effect condition
	#[display(fmt = "Unable to read the first effect condition")]
	EffectConditionFirst( #[error(source)] property::effect_condition::FromBytesError ),
	
	/// Unable to read the second effect condition
	#[display(fmt = "Unable to read the second effect condition")]
	EffectConditionSecond( #[error(source)] property::effect_condition::FromBytesError ),
	
	/// Unable to read the first effect
	#[display(fmt = "Unable to read the first effect")]
	EffectFirst( #[error(source)] property::effect::FromBytesError ),
	
	/// Unable to read the second effect
	#[display(fmt = "Unable to read the second effect")]
	EffectSecond( #[error(source)] property::effect::FromBytesError ),
	
	/// Unable to read the third effect
	#[display(fmt = "Unable to read the third effect")]
	EffectThird( #[error(source)] property::effect::FromBytesError ),
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
	
	/// Unable to write the first effect
	#[display(fmt = "Unable to write the first effect")]
	EffectFirst( #[error(source)] property::effect::ToBytesError ),
	
	/// Unable to write the second effect
	#[display(fmt = "Unable to write the second effect")]
	EffectSecond( #[error(source)] property::effect::ToBytesError ),
	
	/// Unable to write the third effect
	#[display(fmt = "Unable to write the third effect")]
	EffectThird( #[error(source)] property::effect::ToBytesError ),
}


impl Bytes for Item
{
	type ByteArray = [u8; 0xde];
	
	type FromError = FromBytesError;
	fn from_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::FromError>
	{
		// Split bytes
		let bytes = util::array_split!(bytes,
			name                : [0x15],
			unknown_15          : [0x4],
			condition_first     : [0x20],
			condition_second    : [0x20],
			effect_first        : [0x10],
			effect_second       : [0x10],
			effect_third        : [0x10],
			effect_arrow_color  : 1,
			effect_description_0: [0x15],
			effect_description_1: [0x15],
			effect_description_2: [0x15],
			effect_description_3: [0x15],
		);
		
		// And return the struct
		Ok( Self {
			name: util::read_null_ascii_string(bytes.name)
				.map_err(FromBytesError::Name)?
				.chars().collect(),
			
			// Effects
			effect_conditions: [
				Option::<EffectCondition>::from_bytes( bytes.condition_first )
					.map_err(FromBytesError::EffectConditionFirst)?,
				
				Option::<EffectCondition>::from_bytes( bytes.condition_second )
					.map_err(FromBytesError::EffectConditionSecond)?,
			],
			
			effects: [
				Option::<Effect>::from_bytes( bytes.effect_first )
					.map_err(FromBytesError::EffectFirst)?,
					
				Option::<Effect>::from_bytes( bytes.effect_second )
					.map_err(FromBytesError::EffectSecond)?,
					
				Option::<Effect>::from_bytes( bytes.effect_third )
					.map_err(FromBytesError::EffectThird)?,
			],
			
			effect_arrow_color: Option::<ArrowColor>::from_bytes(bytes.effect_arrow_color)
				.map_err(FromBytesError::ArrowColor)?,
			
			effect_description: [
				util::read_null_ascii_string( bytes.effect_description_0 )
					.map_err(FromBytesError::EffectDescriptionFirst)?
					.chars().collect(),
				util::read_null_ascii_string( bytes.effect_description_1 )
					.map_err(FromBytesError::EffectDescriptionSecond)?
					.chars().collect(),
				util::read_null_ascii_string( bytes.effect_description_2 )
					.map_err(FromBytesError::EffectDescriptionThird)?
					.chars().collect(),
				util::read_null_ascii_string( bytes.effect_description_3 )
					.map_err(FromBytesError::EffectDescriptionFourth)?
					.chars().collect(),
			],
			
			// Unknown
			unknown_15: LittleEndian::read_u32(bytes.unknown_15),
		})
	}
	
	type ToError = ToBytesError;
	fn to_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::ToError>
	{
		// Split bytes
		let bytes = util::array_split_mut!(bytes,
			name                : [0x15],
			unknown_15          : [0x4],
			condition_first     : [0x20],
			condition_second    : [0x20],
			effect_first        : [0x10],
			effect_second       : [0x10],
			effect_third        : [0x10],
			effect_arrow_color  : 1,
			effect_description_0: [0x15],
			effect_description_1: [0x15],
			effect_description_2: [0x15],
			effect_description_3: [0x15],
		);
		
		// Name
		util::write_null_ascii_string(self.name.as_ref(), bytes.name)
			.map_err(ToBytesError::Name)?;
		
		// Effect conditions
		self.effect_conditions[0].to_bytes( bytes.condition_first  ).into_ok();
		self.effect_conditions[1].to_bytes( bytes.condition_second ).into_ok();
		
		// Effects
		self.effects[0].to_bytes( bytes.effect_first  ).map_err(ToBytesError::EffectFirst )?;
		self.effects[1].to_bytes( bytes.effect_second ).map_err(ToBytesError::EffectSecond)?;
		self.effects[2].to_bytes( bytes.effect_third  ).map_err(ToBytesError::EffectThird )?;
		
		// Support arrow color
		Option::<ArrowColor>::to_bytes(&self.effect_arrow_color, bytes.effect_arrow_color).into_ok();
		
		// effect_description
		util::write_null_ascii_string(self.effect_description[0].as_ref(), bytes.effect_description_0)
			.map_err(ToBytesError::EffectDescriptionFirst)?;
		util::write_null_ascii_string(self.effect_description[1].as_ref(), bytes.effect_description_1)
			.map_err(ToBytesError::EffectDescriptionSecond)?;
		util::write_null_ascii_string(self.effect_description[2].as_ref(), bytes.effect_description_2)
			.map_err(ToBytesError::EffectDescriptionThird)?;
		util::write_null_ascii_string(self.effect_description[3].as_ref(), bytes.effect_description_3)
			.map_err(ToBytesError::EffectDescriptionFourth)?;
		
		// Unknown
		LittleEndian::write_u32(bytes.unknown_15, self.unknown_15);
		
		// Return Ok
		Ok(())
	}
}
