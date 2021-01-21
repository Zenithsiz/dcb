//! Errors

// Imports
use super::header;

/// Error type for [`Bytes::from_bytes`](dcb_bytes::Bytes::from_bytes)
#[derive(PartialEq, Eq, Clone, Copy, Debug, thiserror::Error)]
pub enum FromBytesError {
	/// Unable to read header
	#[error("Unable to parse header")]
	Header(#[source] header::FromBytesError),
}
