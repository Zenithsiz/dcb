//! Errors

// Imports
use super::header;

/// Error type for [`Sector::new`](super::Sector::new)
#[derive(PartialEq, Eq, Clone, Copy, Debug, thiserror::Error)]
pub enum NewError {
	/// Unable to create address
	#[error("Unable to create address")]
	Address(#[source] header::address::FromSectorPosError),
}

/// Error type for [`Bytes::from_bytes`](dcb_bytes::Bytes::from_bytes)
#[derive(PartialEq, Eq, Clone, Copy, Debug, thiserror::Error)]
pub enum FromBytesError {
	/// Unable to read header
	#[error("Unable to parse header")]
	Header(#[source] header::FromBytesError),

	/// Edc was wrong
	#[error("Found crc {found}, calculated {calculated}")]
	WrongEdc {
		/// Found
		found: u32,

		/// Calculated
		calculated: u32,
	},
}

/// Error type for [`Bytes::to_bytes`](dcb_bytes::Bytes::to_bytes)
#[derive(PartialEq, Eq, Clone, Copy, Debug, thiserror::Error)]
pub enum ToBytesError {
	/// Unable to write header
	#[error("Unable to write header")]
	Header(#[source] header::ToBytesError),
}
