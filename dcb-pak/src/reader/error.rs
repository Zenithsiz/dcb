//! Errors

// Imports
use crate::reader::entry;
use std::io;

/// Error for [`PakFileReader::next_entry`](super::PakFileReader::next_entry)
#[derive(Debug, thiserror::Error)]
pub enum NextEntryError {
	/// Unable to seek to next entry
	#[error("Unable to seek to next entry")]
	SeekNextEntry(#[source] io::Error),

	/// Unable to read entry
	#[error("Unable to read entry")]
	ReadEntry(#[source] entry::FromReaderError),
}
