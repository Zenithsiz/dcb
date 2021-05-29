//! Errors

// Imports
use std::io;

/// Error type for [`Img::deserialize`](super::Img::deserialize)
#[derive(Debug, thiserror::Error)]
pub enum DeserializeError {
	/// Unable to read header
	#[error("Unable to read header")]
	ReadHeader(#[source] io::Error),

	/// Image size didn't match pixels
	#[error("Size didn't match pixels length")]
	SizePixelsMismatch,

	/// Unable to read colors
	#[error("Unable to read colors")]
	ReadColors(#[source] io::Error),
}
