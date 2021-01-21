//! Error

// Imports
use super::{boot, primary, DescriptorKind};

/// Error type for [`Bytes::from_bytes`](dcb_bytes::Bytes::from_bytes)
#[derive(Debug, thiserror::Error)]
pub enum FromBytesError {
	/// Invalid magic
	#[error("Invalid magic {_0:#x?}")]
	InvalidMagic([u8; 5]),

	/// Invalid version
	#[error("Invalid version {_0:#x}")]
	InvalidVersion(u8),

	/// Cannot parse descriptor kind
	#[error("Cannot parse parse descriptor kind {_0:?}")]
	CannotParseKind(DescriptorKind),

	/// Unable to parse boot record
	#[error("Unable to parse boot record")]
	ParseBootRecord(#[source] boot::FromBytesError),

	/// Unable to parse primary
	#[error("Unable to parse primary")]
	ParsePrimary(#[source] primary::FromBytesError),
}
