//! Errors

// Imports
use std::io;

/// Error type for [`Tim::deserialize`](super::Tim::deserialize)
#[derive(Debug, thiserror::Error)]
pub enum DeserializeError {
	/// Unable to read header
	#[error("Unable to read header")]
	ReadHeader(#[source] io::Error),
}
