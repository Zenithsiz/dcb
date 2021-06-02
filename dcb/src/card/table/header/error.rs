//! Errors

// Imports
use super::Header;

/// Error type for [`Bytes::deserialize_bytes`](dcb_bytes::Bytes::deserialize_bytes)
#[derive(PartialEq, Eq, Clone, Copy, Debug, thiserror::Error)]
pub enum DeserializeBytesError {
	/// Magic
	#[error("Magic did not match (found {magic:#x?}, expected {:#x?})", Header::MAGIC)]
	Magic {
		/// Magic that was found
		magic: [u8; 4],
	},
}
