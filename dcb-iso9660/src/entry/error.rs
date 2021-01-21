//! Errors

// Imports
use crate::string;
use dcb_cdrom_xa::ReadSectorError;

/// Error type for [`Bytes::from_bytes`](dcb_bytes::Bytes::from_bytes)
#[derive(Debug, thiserror::Error)]
pub enum FromBytesError {
	/// Record size too small
	#[error("Record size was too small for actual size")]
	RecordSizeTooSmall,

	/// Buffer was too small for header
	#[error("Buffer was too small for header")]
	TooSmallHeader,

	/// Invalid entry flags
	#[error("Invalid entry flags")]
	InvalidFlags,

	/// Buffer was too small for name
	#[error("Buffer was too small for name (expected {_0} for name)")]
	TooSmallName(u8),

	/// Unable to read name
	#[error("Unable to read name")]
	Name(#[source] string::ValidateFileAlphabetError),
}


/// Error type for [`Entry::read`](super::Entry::read)
#[derive(Debug, thiserror::Error)]
pub enum ReadError {
	/// Not a file
	#[error("Not a file")]
	NotAFile,

	/// Unable to read sector
	#[error("Unable to read sector")]
	ReadSector(#[source] ReadSectorError),
}

/// Error type for [`Entry::read_entries`](super::Entry::read_entries)
#[derive(Debug, thiserror::Error)]
pub enum ReadEntriesError {
	/// Not a directory
	#[error("Not a directory")]
	NotADirectory,

	/// Unable to read sector
	#[error("Unable to read sector")]
	ReadSector(#[source] ReadSectorError),

	/// Unable to parse an entry
	#[error("Unable to parse an entry")]
	ParseEntry(#[source] self::FromBytesError),
}
