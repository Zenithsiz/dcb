//! Errors

// Imports
use super::dir;

/// Error for [`Bytes::from_bytes`](super::Bytes::from_bytes)
#[derive(Debug, thiserror::Error)]
pub enum FromBytesError {
	/// Unable to read root directory
	#[error("Unable to read root directory")]
	RootDir(#[source] dir::FromBytesError),
}
