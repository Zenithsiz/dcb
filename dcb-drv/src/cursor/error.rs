//! Errors

use crate::dir::reader::{ReadEntriesError, ReadEntryError};
use std::io;

/// Error for [`DrvFsCursor::new`](super::DrvFsCursor::new)
#[derive(Debug, thiserror::Error)]
pub enum NewError {
	/// Unable to get file size
	#[error("Unable to get file size")]
	FileSize(#[source] io::Error),

	/// Unable to read directory
	#[error("Unable to read directory at {sector_pos:#x}")]
	ReadDir {
		/// Position of the sector
		sector_pos: u32,

		/// Underlying error
		#[source]
		err: ReadEntriesError,
	},

	/// Unable to read directory entry
	#[error("Unable to read directory entry at {sector_pos:#x}")]
	ReadDirEntry {
		/// Position of the sector
		sector_pos: u32,

		/// Underlying error
		#[source]
		err: ReadEntryError,
	},
}

/// Error for [`DrvFsCursor::open_file`](super::DrvFsCursor::open_file)
#[derive(Debug, thiserror::Error)]
pub enum OpenFileError {
	/// Unable to find file
	#[error("Unable to find file")]
	FindFile,

	/// Cannot open directory
	#[error("Cannot open directory")]
	OpenDir,

	/// Attempted to use file as directory
	#[error("Cannot check directory entries of a file")]
	FileDirEntries,

	/// Unable to open file
	#[error("Unable to open file")]
	OpenFile(#[source] io::Error),
}
