//! Errors

// Imports
use crate::string;
use dcb_cdrom_xa::{ReadNthSectorError, SeekSectorError};
use std::io;

/// Error type for [`Bytes::from_bytes`](dcb_bytes::Bytes::from_bytes)
#[derive(Debug, thiserror::Error)]
pub enum FromReaderError {
	/// Unable to read header
	#[error("Unable to read header")]
	ReadHeader(#[source] io::Error),

	/// Record size too small
	#[error("Record size `{_0}` was too small for actual size")]
	RecordSizeTooSmall(u8),

	/// Invalid entry flags
	#[error("Invalid entry flags")]
	InvalidFlags,

	/// Unable to read header
	#[error("Unable to read header")]
	ReadName(#[source] io::Error),

	/// Unable to parse name
	#[error("Unable to parse name")]
	ParseName(#[source] string::ValidateFileAlphabetError),
}


/// Error type for [`Entry::read`](super::Entry::read)
#[derive(Debug, thiserror::Error)]
pub enum ReadFileError {
	/// Not a file
	#[error("Not a file")]
	NotAFile,

	/// Unable to seek to sector
	#[error("Unable to seek to sector")]
	SeekSector(#[source] SeekSectorError),
}

/// Error type for [`Entry::read_entries`](super::Entry::read_entries)
#[derive(Debug, thiserror::Error)]
pub enum ReadDirError {
	/// Not a directory
	#[error("Not a directory")]
	NotADirectory,

	/// Unable to read sector
	#[error("Unable to read sector")]
	ReadSector(#[source] ReadNthSectorError),

	/// Unable to parse an entry
	#[error("Unable to parse an entry")]
	ParseEntry(#[source] self::FromReaderError),
}
