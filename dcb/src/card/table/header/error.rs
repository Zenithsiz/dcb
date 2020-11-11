//! Errors

// Imports
use super::Header;

/// Error type for [`Bytes::from_bytes`](dcb_bytes::Bytes::from_bytes)
#[derive(PartialEq, Eq, Clone, Copy, Debug, thiserror::Error)]
pub enum FromBytesError {
	/// Magic
	#[error("Magic did not match (found {magic:#x?}, expected {:#x?})", Header::MAGIC)]
	Magic {
		/// Magic that was found
		magic: [u8; 4],
	},
}
