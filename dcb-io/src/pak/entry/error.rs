//! Errors

// Imports
use super::animation2d;

/// Error for [`PakEntry::from_reader`](super::PakEntry::from_reader)
#[derive(Debug, thiserror::Error)]
pub enum FromReaderError {
	/// Unable to parse 2d animation
	#[error("Unable to parse 2d animation")]
	ParseAnimation2D(#[source] animation2d::DeserializeError),
}
