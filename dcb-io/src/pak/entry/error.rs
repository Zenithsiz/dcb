//! Errors

// Imports
use crate::pak::header;
use std::io;

/// Error for [`PakEntry::from_reader`](super::PakEntry::from_reader)
#[derive(Debug, thiserror::Error)]
pub enum FromReaderError {
	/// Unable to read header
	#[error("Unable to read header")]
	ReadHeader(#[source] io::Error),

	/// Unable to parse header
	#[error("Unable to parse header")]
	ParseHeader(#[source] header::FromBytesError),
}
