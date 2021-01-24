//! Errors

// Imports
use super::header;
use std::io;

/// Error for [`PakFile::deserialize`](super::PakFile::deserialize)
#[derive(Debug, thiserror::Error)]
pub enum FromReaderError {
	/// Unable to read header
	#[error("Unable to read header")]
	ReadHeader(#[source] io::Error),

	/// Unable to parse header
	#[error("Unable to parse header")]
	ParseHeader(#[source] header::FromBytesError),

	/// Unable to seek past data
	#[error("Unable to seek past entry")]
	SeekPastEntry(#[source] io::Error),
}
