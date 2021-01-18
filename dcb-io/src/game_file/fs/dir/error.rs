//! Errors

// Imports
use super::entry;

/// Error for [`Bytes::from_bytes`](super::Bytes::from_bytes)
#[derive(Debug, thiserror::Error)]
pub enum FromBytesError {
	/// Unable to read entry
	#[error("Unable to read entry")]
	ReadEntry(#[source] entry::FromBytesError),
}
