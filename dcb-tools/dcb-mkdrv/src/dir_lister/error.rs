//! Errors

// Imports
use std::{io, path::PathBuf};

/// Error for [`DirList::new`]
// TODO: Rename
#[derive(Debug, thiserror::Error)]
pub enum DirListNewError {
	/// Unable to read directory
	#[error("Unable to read directory {}", _0.display())]
	ReadDir(PathBuf, #[source] io::Error),

	/// Unable to read entry
	#[error("Unable to read entry in {}", _0.display())]
	ReadEntries(PathBuf, #[source] io::Error),

	/// Too many entries in directory
	#[error("Too many entries in directory")]
	TooManyEntries,
}

/// Error for [`Iterator::Item`]
#[derive(Debug, thiserror::Error)]
pub enum NextError {
	/// Unable to read entry metadata
	#[error("Unable to read entry metadata")]
	ReadMetadata(#[source] io::Error),

	/// Entry had no name
	#[error("Entry had no name")]
	NoEntryName,

	/// Invalid file name
	#[error("Invalid file name")]
	InvalidEntryName(#[source] dcb_util::ascii_str_arr::FromBytesError<0x10>),

	/// File had no file name
	#[error("file had no file name")]
	NoFileExtension,

	/// Invalid extension
	#[error("Invalid extension")]
	InvalidFileExtension(#[source] dcb_util::ascii_str_arr::FromBytesError<0x3>),

	/// Unable to get entry date
	#[error("Unable to get entry date")]
	EntryDate(#[source] io::Error),

	/// Unable to get entry date as time since epoch
	#[error("Unable to get entry date as time since epoch")]
	EntryDateSinceEpoch(#[source] std::time::SystemTimeError),

	/// Unable to get entry date as `i64` seconds since epoch
	#[error("Unable to get entry date as `i64` seconds since epoch")]
	EntryDateI64Secs,

	/// Unable to open file
	#[error("Unable to open file")]
	OpenFile(#[source] io::Error),

	/// Unable to get file size
	#[error("Unable to get file size")]
	FileSize(#[source] io::Error),

	/// File was too big
	#[error("File was too big")]
	FileTooBig,

	/// Unable to open directory
	#[error("Unable to open directory")]
	OpenDir(#[source] self::DirListNewError),
}
