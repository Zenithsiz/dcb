//! Errors

// Imports
use crate::fs::string;

/// Error type for [`Bytes::from_bytes`](dcb_bytes::Bytes::from_bytes)
#[derive(Debug, thiserror::Error)]
pub enum FromBytesError {
	/// Unable to parse system id
	#[error("Unable to parse system id")]
	SystemId(#[source] string::InvalidCharError),

	/// Unable to parse boot id
	#[error("Unable to parse boot id")]
	BootId(#[source] string::InvalidCharError),
}
