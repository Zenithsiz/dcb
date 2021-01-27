//! Errors

// Imports
use super::kind;
/// Error type for [`Bytes::from_bytes`](dcb_bytes::Bytes::from_bytes)
#[derive(Debug, thiserror::Error)]
pub enum FromBytesError {
	/// Unable to parse file kind
	#[error("Unable to parse file kind")]
	Kind(#[source] kind::FromBytesError),
}
