//! Errors

// Imports
use super::kind;
/// Error type for [`Bytes::deserialize_bytes`](dcb_bytes::Bytes::deserialize_bytes)
#[derive(Debug, thiserror::Error)]
pub enum DeserializeBytesError {
	/// Unable to parse file kind
	#[error("Unable to parse file kind")]
	Kind(#[source] kind::DeserializeBytesError),
}
