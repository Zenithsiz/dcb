//! Errors

// Imports
use super::Header;
use crate::util::null_ascii_string;

/// Error type for [`Bytes::from_bytes`](dcb_bytes::Bytes::from_bytes)
#[derive(PartialEq, Eq, Clone, Copy, Debug, thiserror::Error)]
pub enum FromBytesError {
	/// The magic of the table was wrong
	#[error("Found wrong header magic (expected {:?}, found {:?})", Header::MAGIC, magic)]
	Magic {
		/// Magic we found
		magic: [u8; 8],
	},

	/// Unable to read region marker
	#[error("Unable to read the region marker")]
	Name(#[source] null_ascii_string::ReadError),
}

/// Error type for [`Bytes::to_bytes`](dcb_bytes::Bytes::to_bytes)
pub type ToBytesError = !;
