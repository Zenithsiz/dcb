//! Errors

// Imports
use crate::card::property;
use dcb_util::null_ascii_string;

/// Error type for [`Bytes::from_bytes`](dcb_bytes::Bytes::from_bytes)
#[derive(PartialEq, Eq, Clone, Copy, Debug, thiserror::Error)]
pub enum FromBytesError {
	/// Unable to read the digimon name
	#[error("Unable to read the digimon name")]
	Name(#[source] null_ascii_string::ReadError),

	/// Unable to read the first support effect description
	#[error("Unable to read the first line of the effect description")]
	EffectDescription1(#[source] null_ascii_string::ReadError),

	/// Unable to read the second support effect description
	#[error("Unable to read the second line of the effect description")]
	EffectDescription2(#[source] null_ascii_string::ReadError),

	/// Unable to read the third support effect description
	#[error("Unable to read the third line of the effect description")]
	EffectDescription3(#[source] null_ascii_string::ReadError),

	/// Unable to read the fourth support effect description
	#[error("Unable to read the fourth line of the effect description")]
	EffectDescription4(#[source] null_ascii_string::ReadError),

	/// An unknown speciality was found
	#[error("Unknown speciality found")]
	Speciality(#[source] property::speciality::FromBytesError),

	/// An unknown level was found
	#[error("Unknown level found")]
	Level(#[source] property::level::FromBytesError),

	/// An unknown effect arrow color was found
	#[error("Unknown effect arrow color found")]
	ArrowColor(#[source] property::arrow_color::FromBytesError),

	/// An unknown cross move effect was found
	#[error("Unknown cross move effect found")]
	CrossMoveEffect(#[source] property::cross_move_effect::FromBytesError),

	/// Unable to read the circle move
	#[error("Unable to read the circle move")]
	MoveCircle(#[source] property::moves::FromBytesError),

	/// Unable to read the triangle move
	#[error("Unable to read the triangle move")]
	MoveTriangle(#[source] property::moves::FromBytesError),

	/// Unable to read the cross move
	#[error("Unable to read the cross move")]
	MoveCross(#[source] property::moves::FromBytesError),

	/// Unable to read the first effect condition
	#[error("Unable to read the first effect condition")]
	EffectConditionFirst(#[source] property::effect_condition::FromBytesError),

	/// Unable to read the second effect condition
	#[error("Unable to read the second effect condition")]
	EffectConditionSecond(#[source] property::effect_condition::FromBytesError),

	/// Unable to read the first effect
	#[error("Unable to read the first effect")]
	EffectFirst(#[source] property::effect::FromBytesError),

	/// Unable to read the second effect
	#[error("Unable to read the second effect")]
	EffectSecond(#[source] property::effect::FromBytesError),

	/// Unable to read the third effect
	#[error("Unable to read the third effect")]
	EffectThird(#[source] property::effect::FromBytesError),
}

/// Error type for [`Bytes::to_bytes`](dcb_bytes::Bytes::to_bytes)
#[derive(PartialEq, Eq, Clone, Copy, Debug, thiserror::Error)]
#[allow(clippy::pub_enum_variant_names)] // This is a general error, not a specific effect error
pub enum ToBytesError {
	/// Unable to write the first effect
	#[error("Unable to write the first effect")]
	EffectFirst(#[source] property::effect::ToBytesError),

	/// Unable to write the second effect
	#[error("Unable to write the second effect")]
	EffectSecond(#[source] property::effect::ToBytesError),

	/// Unable to write the third effect
	#[error("Unable to write the third effect")]
	EffectThird(#[source] property::effect::ToBytesError),
}
