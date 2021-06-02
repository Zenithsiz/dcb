//! Errors

// Imports
use crate::string;

/// Error type for [`Bytes::deserialize_bytes`](dcb_bytes::Bytes::deserialize_bytes)
#[derive(Debug, thiserror::Error)]
pub enum DeserializeBytesError {
	/// Unable to parse system id
	#[error("Unable to parse system id")]
	SystemId(#[source] string::InvalidCharError),

	/// Unable to parse boot id
	#[error("Unable to parse boot id")]
	BootId(#[source] string::InvalidCharError),
}
