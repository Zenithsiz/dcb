//! Errors

// Imports
use super::submode;

/// Error type for [`Bytes::from_bytes`](dcb_bytes::Bytes::from_bytes)
#[derive(PartialEq, Eq, Clone, Copy, Debug, thiserror::Error)]
pub enum FromBytesError {
	/// Unable to read submode
	#[error("Unable to read submode")]
	SubMode(submode::BytesError),
}

/// Error type for [`Bytes::to_bytes`](dcb_bytes::Bytes::to_bytes)
#[derive(PartialEq, Eq, Clone, Copy, Debug, thiserror::Error)]
pub enum ToBytesError {
	/// Unable to write submode
	#[error("Unable to write submode")]
	SubMode(submode::BytesError),
}
