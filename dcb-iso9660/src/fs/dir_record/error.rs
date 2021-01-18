//! Errors

// Imports
use crate::fs::string;

/// Error type for [`Bytes::from_bytes`](dcb_bytes::Bytes::from_bytes)
#[derive(Debug, thiserror::Error)]
pub enum FromBytesError {
	/// Too small
	#[error("Buffer was too small for header")]
	TooSmallHeader,

	/// Too small
	#[error("Buffer was too small for name (expected {_0} for name)")]
	TooSmallName(u8),

	/// Unable to read name
	#[error("Unable to read name")]
	Name(#[source] string::ValidateFileAlphabetError),
}
