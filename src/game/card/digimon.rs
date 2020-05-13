#![doc(include = "digimon.md")]

// byteorder
use byteorder::{ByteOrder, LittleEndian};

// Crate
use crate::game::{
	bytes::Validation,
	card::property::{self, ArrowColor, CrossMoveEffect, Effect, EffectCondition, Level, Move, Speciality},
	util, Bytes,
};

/// A digimon card
///
/// Contains all information about each digimon card stored in the [`Card Table`](crate::game::card::table::Table)
#[derive(PartialEq, Eq, Clone, Hash, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Digimon {
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

	/// The digimon's circle move
	pub move_circle: Move,

	/// The digimon's triangle move
	pub move_triangle: Move,

	/// The digimon's cross move
	pub move_cross: Move,

	/// The digimon's cross move effect, if any
	#[serde(default)]
	pub cross_move_effect: Option<CrossMoveEffect>,

	/// The effect's description.
	///
	/// The description is split along 4 lines, each
	/// being an ascii string with 20 characters at most.
	pub effect_description: [ascii::AsciiString; 4],

	/// The effect's arrow color
	#[serde(default)]
	pub effect_arrow_color: Option<ArrowColor>,

	/// The effect's conditions
	#[serde(default)]
	pub effect_conditions: [Option<EffectCondition>; 2],

	/// The effects
	#[serde(default)]
	pub effects: [Option<Effect>; 3],

	// Unknown fields
	pub unknown_1a: u8,
	pub unknown_15: u16,
	pub unknown_e2: u8,
}

/// Error type for [`Bytes::from_bytes`]
#[derive(Debug)]
#[derive(derive_more::Display, err_impl::Error)]
pub enum FromBytesError {
	/// Unable to read the digimon name
	#[display(fmt = "Unable to read the digimon name")]
	Name(#[error(source)] util::ReadNullAsciiStringError),

	/// Unable to read the first support effect description
	#[display(fmt = "Unable to read the first line of the effect description")]
	EffectDescriptionFirst(#[error(source)] util::ReadNullAsciiStringError),

	/// Unable to read the second support effect description
	#[display(fmt = "Unable to read the second line of the effect description")]
	EffectDescriptionSecond(#[error(source)] util::ReadNullAsciiStringError),

	/// Unable to read the third support effect description
	#[display(fmt = "Unable to read the third line of the effect description")]
	EffectDescriptionThird(#[error(source)] util::ReadNullAsciiStringError),

	/// Unable to read the fourth support effect description
	#[display(fmt = "Unable to read the fourth line of the effect description")]
	EffectDescriptionFourth(#[error(source)] util::ReadNullAsciiStringError),

	/// An unknown speciality was found
	#[display(fmt = "Unknown speciality found")]
	Speciality(#[error(source)] property::speciality::FromBytesError),

	/// An unknown level was found
	#[display(fmt = "Unknown level found")]
	Level(#[error(source)] property::level::FromBytesError),

	/// An unknown effect arrow color was found
	#[display(fmt = "Unknown effect arrow color found")]
	ArrowColor(#[error(source)] property::arrow_color::FromBytesError),

	/// An unknown cross move effect was found
	#[display(fmt = "Unknown cross move effect found")]
	CrossMoveEffect(#[error(source)] property::cross_move_effect::FromBytesError),

	/// Unable to read the circle move
	#[display(fmt = "Unable to read the circle move")]
	MoveCircle(#[error(source)] property::moves::FromBytesError),

	/// Unable to read the triangle move
	#[display(fmt = "Unable to read the triangle move")]
	MoveTriangle(#[error(source)] property::moves::FromBytesError),

	/// Unable to read the cross move
	#[display(fmt = "Unable to read the cross move")]
	MoveCross(#[error(source)] property::moves::FromBytesError),

	/// Unable to read the first effect condition
	#[display(fmt = "Unable to read the first effect condition")]
	EffectConditionFirst(#[error(source)] property::effect_condition::FromBytesError),

	/// Unable to read the second effect condition
	#[display(fmt = "Unable to read the second effect condition")]
	EffectConditionSecond(#[error(source)] property::effect_condition::FromBytesError),

	/// Unable to read the first effect
	#[display(fmt = "Unable to read the first effect")]
	EffectFirst(#[error(source)] property::effect::FromBytesError),

	/// Unable to read the second effect
	#[display(fmt = "Unable to read the second effect")]
	EffectSecond(#[error(source)] property::effect::FromBytesError),

	/// Unable to read the third effect
	#[display(fmt = "Unable to read the third effect")]
	EffectThird(#[error(source)] property::effect::FromBytesError),
}

/// Error type for [`Bytes::to_bytes`]
#[derive(Debug)]
#[derive(derive_more::Display, err_impl::Error)]
pub enum ToBytesError {
	/// Unable to write the digimon name
	#[display(fmt = "Unable to write the digimon name")]
	Name(#[error(source)] util::WriteNullAsciiStringError),

	/// Unable to write the first support effect description
	#[display(fmt = "Unable to write the first line of the effect description")]
	EffectDescriptionFirst(#[error(source)] util::WriteNullAsciiStringError),

	/// Unable to write the second support effect description
	#[display(fmt = "Unable to write the second line of the effect description")]
	EffectDescriptionSecond(#[error(source)] util::WriteNullAsciiStringError),

	/// Unable to write the third support effect description
	#[display(fmt = "Unable to write the third line of the effect description")]
	EffectDescriptionThird(#[error(source)] util::WriteNullAsciiStringError),

	/// Unable to write the fourth support effect description
	#[display(fmt = "Unable to write the fourth line of the effect description")]
	EffectDescriptionFourth(#[error(source)] util::WriteNullAsciiStringError),

	/// Unable to write the circle move
	#[display(fmt = "Unable to write the circle move")]
	MoveCircle(#[error(source)] property::moves::ToBytesError),

	/// Unable to write the triangle move
	#[display(fmt = "Unable to write the triangle move")]
	MoveTriangle(#[error(source)] property::moves::ToBytesError),

	/// Unable to write the cross move
	#[display(fmt = "Unable to write the cross move")]
	MoveCross(#[error(source)] property::moves::ToBytesError),

	/// Unable to write the first effect
	#[display(fmt = "Unable to write the first effect")]
	EffectFirst(#[error(source)] property::effect::ToBytesError),

	/// Unable to write the second effect
	#[display(fmt = "Unable to write the second effect")]
	EffectSecond(#[error(source)] property::effect::ToBytesError),

	/// Unable to write the third effect
	#[display(fmt = "Unable to write the third effect")]
	EffectThird(#[error(source)] property::effect::ToBytesError),
}

impl Bytes for Digimon {
	type ByteArray = [u8; 0x138];
	type FromError = FromBytesError;
	type ToError = ToBytesError;

	fn from_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::FromError> {
		// Split bytes
		let bytes = util::array_split!(bytes,
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

		// Return the struct after building it
		Ok(Self {
			name: util::read_null_ascii_string(bytes.name).map_err(FromBytesError::Name)?.to_ascii_string(),

			speciality: Speciality::from_bytes(&((bytes.speciality_level & 0xF0) >> 4)).map_err(FromBytesError::Speciality)?,

			level: Level::from_bytes(&((bytes.speciality_level & 0x0F) >> 0)).map_err(FromBytesError::Level)?,

			dp_cost: *bytes.dp_cost,
			dp_give: *bytes.dp_give,

			hp: LittleEndian::read_u16(bytes.hp),

			// Moves
			move_circle:   Move::from_bytes(bytes.move_circle).map_err(FromBytesError::MoveCircle)?,
			move_triangle: Move::from_bytes(bytes.move_triangle).map_err(FromBytesError::MoveTriangle)?,
			move_cross:    Move::from_bytes(bytes.move_cross).map_err(FromBytesError::MoveCross)?,

			// Effects
			effect_conditions: [
				Option::<EffectCondition>::from_bytes(bytes.condition_first).map_err(FromBytesError::EffectConditionFirst)?,
				Option::<EffectCondition>::from_bytes(bytes.condition_second).map_err(FromBytesError::EffectConditionSecond)?,
			],

			effects: [
				Option::<Effect>::from_bytes(bytes.effect_first).map_err(FromBytesError::EffectFirst)?,
				Option::<Effect>::from_bytes(bytes.effect_second).map_err(FromBytesError::EffectSecond)?,
				Option::<Effect>::from_bytes(bytes.effect_third).map_err(FromBytesError::EffectThird)?,
			],

			cross_move_effect: Option::<CrossMoveEffect>::from_bytes(bytes.cross_move_effect).map_err(FromBytesError::CrossMoveEffect)?,

			effect_arrow_color: Option::<ArrowColor>::from_bytes(bytes.effect_arrow_color).map_err(FromBytesError::ArrowColor)?,

			effect_description: [
				util::read_null_ascii_string(bytes.effect_description_0)
					.map_err(FromBytesError::EffectDescriptionFirst)?
					.to_ascii_string(),
				util::read_null_ascii_string(bytes.effect_description_1)
					.map_err(FromBytesError::EffectDescriptionSecond)?
					.to_ascii_string(),
				util::read_null_ascii_string(bytes.effect_description_2)
					.map_err(FromBytesError::EffectDescriptionThird)?
					.to_ascii_string(),
				util::read_null_ascii_string(bytes.effect_description_3)
					.map_err(FromBytesError::EffectDescriptionFourth)?
					.to_ascii_string(),
			],

			// Unknown
			unknown_15: LittleEndian::read_u16(bytes.unknown_15),
			unknown_1a: *bytes.unknown_1a,
			unknown_e2: *bytes.unknown_e2,
		})
	}

	fn to_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::ToError> {
		// Split bytes
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

		// Name
		util::write_null_ascii_string(self.name.as_ref(), bytes.name).map_err(ToBytesError::Name)?;

		// Speciality / Level
		{
			let (mut speciality_byte, mut level_byte) = (0u8, 0u8);

			// Note: Buffers have 1 byte, so this can't fail
			self.speciality.to_bytes(&mut speciality_byte)?;
			self.level.to_bytes(&mut level_byte)?;

			// Merge them
			*bytes.speciality_level = (speciality_byte << 4) | level_byte;
		}

		// DP / +P
		*bytes.dp_cost = self.dp_cost;
		*bytes.dp_give = self.dp_give;

		// Health
		LittleEndian::write_u16(bytes.hp, self.hp);

		// Moves
		self.move_circle.to_bytes(bytes.move_circle).map_err(ToBytesError::MoveCircle)?;
		self.move_triangle.to_bytes(bytes.move_triangle).map_err(ToBytesError::MoveTriangle)?;
		self.move_cross.to_bytes(bytes.move_cross).map_err(ToBytesError::MoveCross)?;

		// Effects
		self.effect_conditions[0].to_bytes(bytes.condition_first).into_ok();
		self.effect_conditions[1].to_bytes(bytes.condition_second).into_ok();

		self.effects[0].to_bytes(bytes.effect_first).map_err(ToBytesError::EffectFirst)?;
		self.effects[1].to_bytes(bytes.effect_second).map_err(ToBytesError::EffectSecond)?;
		self.effects[2].to_bytes(bytes.effect_third).map_err(ToBytesError::EffectThird)?;

		Option::<CrossMoveEffect>::to_bytes(&self.cross_move_effect, bytes.cross_move_effect).into_ok();

		Option::<ArrowColor>::to_bytes(&self.effect_arrow_color, bytes.effect_arrow_color).into_ok();

		util::write_null_ascii_string(self.effect_description[0].as_ref(), bytes.effect_description_0)
			.map_err(ToBytesError::EffectDescriptionFirst)?;
		util::write_null_ascii_string(self.effect_description[1].as_ref(), bytes.effect_description_1)
			.map_err(ToBytesError::EffectDescriptionSecond)?;
		util::write_null_ascii_string(self.effect_description[2].as_ref(), bytes.effect_description_2)
			.map_err(ToBytesError::EffectDescriptionThird)?;
		util::write_null_ascii_string(self.effect_description[3].as_ref(), bytes.effect_description_3)
			.map_err(ToBytesError::EffectDescriptionFourth)?;

		// Unknown
		LittleEndian::write_u16(bytes.unknown_15, self.unknown_15);
		*bytes.unknown_1a = self.unknown_1a;
		*bytes.unknown_e2 = self.unknown_e2;

		// Return Ok
		Ok(())
	}

	fn validate(&self) -> Validation {
		Validation::new()
	}
}
