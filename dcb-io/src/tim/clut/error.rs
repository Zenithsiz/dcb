//! Errors

// Imports
use std::io;

/// Error for [`Clut::deserialize`](super::Clut::deserialize)
#[derive(Debug, thiserror::Error)]
pub enum DeserializeError {
	/// Unable to read header
	#[error("Unable to read header")]
	ReadHeader(#[source] io::Error),

	/// Length was invalid
	#[error("Length was invalid")]
	InvalidLength,

	/// Unable to read all colors
	#[error("Unable to read all colors")]
	ReadColors(#[source] io::Error),
}
