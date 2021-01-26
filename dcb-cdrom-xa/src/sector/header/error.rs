//! Errors

// Imports
use super::address;

/// Error type for [`Bytes::from_bytes`](dcb_bytes::Bytes::from_bytes)
#[derive(PartialEq, Eq, Clone, Copy, Debug, thiserror::Error)]
pub enum FromBytesError {
	/// Sync was wrong
	#[error("Sync was wrong, found {_0:?}")]
	WrongSync([u8; 0xc]),

	/// Invalid mode
	#[error("Invalid mode {_0:?}")]
	InvalidMode(u8),

	/// Unable to read address
	#[error("Unable to write address")]
	Address(address::FromBytesError),
}

/// Error type for [`Bytes::to_bytes`](dcb_bytes::Bytes::to_bytes)
#[derive(PartialEq, Eq, Clone, Copy, Debug, thiserror::Error)]
pub enum ToBytesError {
	/// Unable to write address
	#[error("Unable to write address")]
	Address(address::ToBytesError),
}
