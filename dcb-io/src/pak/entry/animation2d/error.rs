//! Errors

// Imports
use dcb_util::null_ascii_string;
use std::io;

/// Error for [`Animation2d::deserialize`](super::Animation2d::deserialize)
#[derive(Debug, thiserror::Error)]
pub enum DeserializeError {
	/// Unable to read header
	#[error("Unable to read header")]
	ReadHeader(#[source] io::Error),

	/// Unable to parse name
	#[error("Unable to parse name")]
	ParseName(#[source] null_ascii_string::ReadError),

	/// Unable to read frame
	#[error("Unable to read frame")]
	ReadFrame(#[source] io::Error),
}
