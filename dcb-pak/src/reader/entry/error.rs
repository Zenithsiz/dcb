//! Errors

// Imports
use crate::header;
use std::io;

/// Error for [`PakEntryReader::from_reader`](super::PakEntryReader::from_reader)
#[derive(Debug, thiserror::Error)]
pub enum FromReaderError {
	/// Unable to read header
	#[error("Unable to read header")]
	ReadHeader(#[source] io::Error),

	/// Unable to parse header
	#[error("Unable to parse header")]
	ParseHeader(#[source] header::FromBytesError),
}
