//! Errors

// Imports
use dcb_util::ascii_str_arr;

/// Error for [`DirEntryReader::from_bytes`](super::DirEntryReader::from_bytes)
#[derive(Debug, thiserror::Error)]
pub enum FromBytesError {
	/// Invalid kind
	#[error("Invalid kind {_0:#x}")]
	InvalidKind(u8),

	/// Unable to read name
	#[error("Unable to read name")]
	Name(#[source] ascii_str_arr::FromBytesError<0x10>),

	/// Unable to read extension
	#[error("Unable to read extension")]
	Extension(#[source] ascii_str_arr::FromBytesError<0x3>),
}
