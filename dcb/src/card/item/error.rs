//! Errors

// Imports
use crate::card::property;
use zutil::null_ascii_string;

/// Error type for [`Bytes::deserialize_bytes`](dcb_bytes::Bytes::deserialize_bytes)
#[derive(PartialEq, Eq, Clone, Copy, Debug, thiserror::Error)]
pub enum DeserializeBytesError {
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

	/// An unknown effect arrow color was found
	#[error("Unknown effect arrow color found")]
	ArrowColor(#[source] property::arrow_color::DeserializeBytesError),

	/// Unable to read the first effect condition
	#[error("Unable to read the first effect condition")]
	EffectConditionFirst(#[source] property::effect_condition::DeserializeBytesError),

	/// Unable to read the second effect condition
	#[error("Unable to read the second effect condition")]
	EffectConditionSecond(#[source] property::effect_condition::DeserializeBytesError),

	/// Unable to read the first effect
	#[error("Unable to read the first effect")]
	EffectFirst(#[source] property::effect::DeserializeBytesError),

	/// Unable to read the second effect
	#[error("Unable to read the second effect")]
	EffectSecond(#[source] property::effect::DeserializeBytesError),

	/// Unable to read the third effect
	#[error("Unable to read the third effect")]
	EffectThird(#[source] property::effect::DeserializeBytesError),
}

/// Error type for [`Bytes::serialize_bytes`](dcb_bytes::Bytes::serialize_bytes)
#[derive(PartialEq, Eq, Clone, Copy, Debug, thiserror::Error)]
#[allow(clippy::enum_variant_names)] // This is a general error, not a specific effect error
pub enum SerializeBytesError {
	/// Unable to write the first effect
	#[error("Unable to write the first effect")]
	EffectFirst(#[source] property::effect::SerializeBytesError),

	/// Unable to write the second effect
	#[error("Unable to write the second effect")]
	EffectSecond(#[source] property::effect::SerializeBytesError),

	/// Unable to write the third effect
	#[error("Unable to write the third effect")]
	EffectThird(#[source] property::effect::SerializeBytesError),
}
