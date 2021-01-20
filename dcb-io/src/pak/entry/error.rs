//! Errors

// Imports
use dcb_util::null_ascii_string;

/// Error for [`PakEntry::deserialize`](super::PakEntry::deserialize)
#[derive(Debug, thiserror::Error)]
pub enum DeserializeError {
	/// Missing name
	#[error("Missing name")]
	MissingName,

	/// Unable to parse name
	#[error("Unable to parse name")]
	ParseName(#[source] null_ascii_string::ReadError),
}
