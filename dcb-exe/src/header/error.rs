//! Errors

// Imports
use super::Header;
use dcb_util::null_ascii_string;

/// Error type for [`Bytes::deserialize_bytes`](dcb_bytes::Bytes::deserialize_bytes)
#[derive(PartialEq, Eq, Clone, Copy, Debug, thiserror::Error)]
pub enum DeserializeBytesError {
	/// The magic of the table was wrong
	#[error("Found wrong header magic (expected {:?}, found {:?})", Header::MAGIC, magic)]
	Magic {
		/// Magic we found
		magic: [u8; 8],
	},

	/// Size wasn't a multiple of 0x800
	#[error("Size {size} wasn't a multiple of 0x800")]
	SizeAlignment {
		/// Size
		size: u32,
	},

	/// Unable to read region marker
	#[error("Unable to read the region marker")]
	Name(#[source] null_ascii_string::ReadError),
}

/// Error type for [`Bytes::serialize_bytes`](dcb_bytes::Bytes::serialize_bytes)
pub type SerializeBytesError = !;
