//! Errors

// Imports
use super::entry;
use std::io;

/// Error for [`Dir::from_reader`](super::Dir::from_reader)
#[derive(Debug, thiserror::Error)]
pub enum FromReaderError {
	/// Unable to read entry bytes
	#[error("Unable to read entry bytes")]
	ReadEntry(#[source] io::Error),

	/// Unable to parse entry
	#[error("Unable to parse entry")]
	ParseEntry(#[source] entry::FromBytesError),
}
