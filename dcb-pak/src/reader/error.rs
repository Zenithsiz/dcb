//! Errors

// Imports
use crate::entry;
use std::io;

/// Error for [`PakFile::deserialize`](super::PakFile::deserialize)
#[derive(Debug, thiserror::Error)]
pub enum FromReaderError {
	/// Unable to seek to next entry
	#[error("Unable to seek to next entry")]
	SeekNextEntry(#[source] io::Error),

	/// Unable to read entry
	#[error("Unable to read entry")]
	ReadEntry(#[source] entry::FromReaderError),
}
