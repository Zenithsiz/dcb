//! Errors

// Imports
use super::animation2d;

/// Error for [`PakEntry::deserialize`](super::PakEntry::deserialize)
#[derive(Debug, thiserror::Error)]
pub enum DeserializeError {
	/// Unable to parse 2d animation
	#[error("Unable to parse 2d animation")]
	ParseAnimation2D(#[source] animation2d::DeserializeError),
}
