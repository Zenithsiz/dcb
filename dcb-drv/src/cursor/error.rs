//! Errors

// Imports
use crate::ptr::{FileCursorError, ReadEntriesError, ReadEntryError};
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

/// Error for [`DrvFsCursor::find`](super::DrvFsCursor::find)
#[derive(Debug, thiserror::Error)]
pub enum FindError {
	/// Path was empty
	#[error("Path was empty")]
	EmptyPath,

	/// Unable to find file
	#[error("Unable to find file")]
	FindFile,

	/// Attempted to use file as directory
	#[error("Cannot check directory entries of a file")]
	FileDirEntries,
}

/// Error for [`DrvFsCursor::open_file`](super::DrvFsCursor::open_file)
#[derive(Debug, thiserror::Error)]
pub enum OpenFileError {
	/// Unable to find entry
	#[error("Unable to find file")]
	FindFile(#[source] FindError),

	/// Cannot open directory
	#[error("Cannot open directory")]
	OpenDir,

	/// Unable to open file
	#[error("Unable to open file")]
	OpenFile(#[source] FileCursorError),
}

/// Error for [`DrvFsCursor::swap_files`](super::DrvFsCursor::swap_files)
#[derive(Debug, thiserror::Error)]
pub enum SwapFilesError {
	/// Both paths were equal
	#[error("Both paths were equal")]
	BothPathsEqual,

	/// Unable to find common path
	#[error("Unable to find common path")]
	FindCommonPath(#[source] FindError),

	/// Cannot swap directories
	#[error("Cannot swap directories")]
	SwapDirs,

	/// Common path was a file
	#[error("Common path was a file")]
	CommonPathFile,

	/// Unable to find lhs common dir entry
	#[error("Unable to find lhs common dir entry")]
	CommonPathLhsEntry,

	/// Unable to find rhs common dir entry
	#[error("Unable to find rhs common dir entry")]
	CommonPathRhsEntry,

	/// Attempted to use file as directory
	#[error("Cannot check lhs directory entries of a file")]
	LhsFileDirEntries,

	/// Attempted to use file as directory
	#[error("Cannot check rhs directory entries of a file")]
	RhsFileDirEntries,

	/// Unable to find lhs
	#[error("Unable to find lhs")]
	FindLhs(#[source] FindError),

	/// Unable to find rhs
	#[error("Unable to find rhs")]
	FindRhs(#[source] FindError),

	/// Unable to find lhs file
	#[error("Unable to find lhs file")]
	FindLhsFile,

	/// Unable to find rhs file
	#[error("Unable to find rhs file")]
	FindRhsFile,

	/// Unable to seek to lhs file entry
	#[error("Seek lhs entry")]
	SeekLhsEntry(#[source] io::Error),

	/// Unable to seek to rhs file entry
	#[error("Seek rhs entry")]
	SeekRhsEntry(#[source] io::Error),

	/// Unable to write lhs file entry
	#[error("Write lhs entry")]
	WriteLhsEntry(#[source] io::Error),

	/// Unable to seek to rhs file entry
	#[error("Write rhs entry")]
	WriteRhsEntry(#[source] io::Error),
}
