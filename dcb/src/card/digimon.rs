#![doc(include = "digimon.md")]

// Modules
pub mod error;

// Exports
pub use error::{FromBytesError, ToBytesError};

// Imports
use crate::card::property::{
	ArrowColor, CrossMoveEffect, Effect, EffectCondition, Level, MaybeArrowColor, MaybeCrossMoveEffect, MaybeEffect, MaybeEffectCondition, Move,
	Speciality,
};
use dcb_util::{array_split, array_split_mut, null_ascii_string::NullAsciiString, AsciiStrArr};

use byteorder::{ByteOrder, LittleEndian};
use dcb_bytes::Bytes;
use ref_cast::RefCast;

/// A digimon card
///
/// Contains all information about each digimon card stored in the [`Card Table`](crate::card::table::Table)
#[derive(PartialEq, Eq, Clone, Hash, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Digimon {
	/// The digimon's name
	pub name: AsciiStrArr<0x14>,

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
	/// The description is split along 4 lines
	pub effect_description: [AsciiStrArr<0x14>; 4],

	/// The effect's arrow color
	#[serde(default)]
	pub effect_arrow_color: Option<ArrowColor>,

	/// The effect's conditions
	#[serde(default)]
	pub effect_conditions: [Option<EffectCondition>; 2],

	/// The effects
	#[serde(default)]
	pub effects: [Option<Effect>; 3],

	/// Unknown field at `0x1a`
	pub unknown_1a: u8,

	/// Unknown field at `0x15`
	pub unknown_15: u16,

	/// Unknown field at `0xe2`
	pub unknown_e2: u8,
}

impl Bytes for Digimon {
	type ByteArray = [u8; 0x138];
	type FromError = FromBytesError;
	type ToError = ToBytesError;

	fn from_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::FromError> {
		// Split bytes
		let bytes = array_split!(bytes,
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
			name: NullAsciiString::read_string(bytes.name).map_err(FromBytesError::Name)?,

			speciality: Speciality::from_bytes(&((bytes.speciality_level & 0xF0) >> 4)).map_err(FromBytesError::Speciality)?,

			level: Level::from_bytes(&(bytes.speciality_level & 0x0F)).map_err(FromBytesError::Level)?,

			dp_cost: *bytes.dp_cost,
			dp_give: *bytes.dp_give,

			hp: LittleEndian::read_u16(bytes.hp),

			// Moves
			move_circle:   Move::from_bytes(bytes.move_circle).map_err(FromBytesError::MoveCircle)?,
			move_triangle: Move::from_bytes(bytes.move_triangle).map_err(FromBytesError::MoveTriangle)?,
			move_cross:    Move::from_bytes(bytes.move_cross).map_err(FromBytesError::MoveCross)?,

			// Effects
			effect_conditions: [
				MaybeEffectCondition::from_bytes(bytes.condition_first)
					.map_err(FromBytesError::EffectConditionFirst)?
					.into(),
				MaybeEffectCondition::from_bytes(bytes.condition_second)
					.map_err(FromBytesError::EffectConditionSecond)?
					.into(),
			],

			effects: [
				MaybeEffect::from_bytes(bytes.effect_first).map_err(FromBytesError::EffectFirst)?.into(),
				MaybeEffect::from_bytes(bytes.effect_second).map_err(FromBytesError::EffectSecond)?.into(),
				MaybeEffect::from_bytes(bytes.effect_third).map_err(FromBytesError::EffectThird)?.into(),
			],

			cross_move_effect: MaybeCrossMoveEffect::from_bytes(bytes.cross_move_effect)
				.map_err(FromBytesError::CrossMoveEffect)?
				.into(),

			effect_arrow_color: MaybeArrowColor::from_bytes(bytes.effect_arrow_color)
				.map_err(FromBytesError::ArrowColor)?
				.into(),

			effect_description: [
				bytes.effect_description_0.read_string().map_err(FromBytesError::EffectDescription1)?,
				bytes.effect_description_1.read_string().map_err(FromBytesError::EffectDescription2)?,
				bytes.effect_description_2.read_string().map_err(FromBytesError::EffectDescription3)?,
				bytes.effect_description_3.read_string().map_err(FromBytesError::EffectDescription4)?,
			],

			// Unknown
			unknown_15: LittleEndian::read_u16(bytes.unknown_15),
			unknown_1a: *bytes.unknown_1a,
			unknown_e2: *bytes.unknown_e2,
		})
	}

	fn to_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::ToError> {
		// Split bytes
		let bytes = array_split_mut!(bytes,
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
		bytes.name.write_string(&self.name);

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
		self.move_circle.to_bytes(bytes.move_circle).into_ok();
		self.move_triangle.to_bytes(bytes.move_triangle).into_ok();
		self.move_cross.to_bytes(bytes.move_cross).into_ok();

		// Effects
		MaybeEffectCondition::ref_cast(&self.effect_conditions[0])
			.to_bytes(bytes.condition_first)
			.into_ok();
		MaybeEffectCondition::ref_cast(&self.effect_conditions[1])
			.to_bytes(bytes.condition_second)
			.into_ok();

		MaybeEffect::ref_cast(&self.effects[0])
			.to_bytes(bytes.effect_first)
			.map_err(ToBytesError::EffectFirst)?;
		MaybeEffect::ref_cast(&self.effects[1])
			.to_bytes(bytes.effect_second)
			.map_err(ToBytesError::EffectSecond)?;
		MaybeEffect::ref_cast(&self.effects[2])
			.to_bytes(bytes.effect_third)
			.map_err(ToBytesError::EffectThird)?;

		MaybeCrossMoveEffect::ref_cast(&self.cross_move_effect)
			.to_bytes(bytes.cross_move_effect)
			.into_ok();

		MaybeArrowColor::ref_cast(&self.effect_arrow_color)
			.to_bytes(bytes.effect_arrow_color)
			.into_ok();

		bytes.effect_description_0.write_string(&self.effect_description[0]);
		bytes.effect_description_1.write_string(&self.effect_description[1]);
		bytes.effect_description_2.write_string(&self.effect_description[2]);
		bytes.effect_description_3.write_string(&self.effect_description[3]);

		// Unknown
		LittleEndian::write_u16(bytes.unknown_15, self.unknown_15);
		*bytes.unknown_1a = self.unknown_1a;
		*bytes.unknown_e2 = self.unknown_e2;

		// Return Ok
		Ok(())
	}
}
