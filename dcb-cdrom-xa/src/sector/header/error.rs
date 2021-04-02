//! Errors

// Imports
use super::{address, subheader, SubHeader};

/// Error type for [`Bytes::from_bytes`](dcb_bytes::Bytes::from_bytes)
#[derive(PartialEq, Eq, Clone, Copy, Debug, thiserror::Error)]
pub enum FromBytesError {
	/// Sync was wrong
	#[error("Sync was wrong, found {_0:?}")]
	WrongSync([u8; 0xc]),

	/// Invalid mode
	#[error("Invalid mode {_0:?}")]
	InvalidMode(u8),

	/// Unable to read subheader
	#[error("Unable to parse subheader")]
	SubHeader(#[source] subheader::FromBytesError),

	/// The two sub-headers were different
	#[error("The two sub-headers were different {_0:?} & {_1:?}")]
	DifferentSubHeaders(SubHeader, SubHeader),

	/// Unable to read address
	#[error("Unable to parse address")]
	Address(#[source] address::FromBytesError),
}

/// Error type for [`Bytes::to_bytes`](dcb_bytes::Bytes::to_bytes)
#[derive(PartialEq, Eq, Clone, Copy, Debug, thiserror::Error)]
pub enum ToBytesError {
	/// Unable to write subheader
	#[error("Unable to write subheader")]
	SubHeader(subheader::ToBytesError),

	/// Unable to write address
	#[error("Unable to write address")]
	Address(address::ToBytesError),
}
