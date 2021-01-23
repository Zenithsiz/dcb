//! Errors

// Imports
use super::{entry, header};
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

	/// Unable to get stream position
	#[error("Unable to get stream position")]
	GetStreamPos(#[source] io::Error),

	/// Unable to parse entry
	#[error("Unable to parse entry")]
	ParseEntry(#[source] entry::FromReaderError),

	/// Unable to set stream position
	#[error("Unable to set stream position")]
	SetStreamPos(#[source] io::Error),
}
