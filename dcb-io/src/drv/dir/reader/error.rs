//! Errors

// Imports
use std::io;
use crate::drv::dir::entry;

/// Error for [`DirReader::read_entries`](super::DirReader::read_entries)
#[derive(Debug, thiserror::Error)]
pub enum ReadEntriesError {
	/// Unable to seek to directory
	#[error("Unable to seek to directory")]
	Seek(#[source] io::Error),
}

/// Error for [`DirReader::read_entries`](super::DirReader::read_entries)
#[derive(Debug, thiserror::Error)]
pub enum ReadEntryError {
	/// Unable to read entry bytes
	#[error("Unable to read entry bytes")]
	ReadEntry(#[source] io::Error),

	/// Unable to parse entry
	#[error("Unable to parse entry")]
	ParseEntry(#[source] entry::reader::FromBytesError),
}
