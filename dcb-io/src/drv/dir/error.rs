//! Errors

// Imports
use super::entry;
use std::io;

/// Error for [`Dir::entries`](super::Dir::entries)
#[derive(Debug, thiserror::Error)]
pub enum EntriesError {
	/// Unable to seek to directory
	#[error("Unable to seek to directory")]
	Seek(#[source] io::Error),
}

/// Error for [`Dir::entries`](super::Dir::entries)
#[derive(Debug, thiserror::Error)]
pub enum ReadEntryError {
	/// Unable to read entry bytes
	#[error("Unable to read entry bytes")]
	ReadEntry(#[source] io::Error),

	/// Unable to parse entry
	#[error("Unable to parse entry")]
	ParseEntry(#[source] entry::FromBytesError),
}
