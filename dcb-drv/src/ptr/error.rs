//! Errors

// Imports
use crate::PathBuf;
use std::io;

/// Error for [`FilePtr::cursor`](super::FilePtr::cursor)
#[derive(Debug, thiserror::Error)]
pub enum FileCursorError {
	/// Unable to seek to file
	#[error("Unable to seek to file")]
	Seek(#[source] io::Error),
}

/// Error for [`DirPtr::read_entries`](super::DirPtr::read_entries)
#[derive(Debug, thiserror::Error)]
pub enum ReadEntriesError {
	/// Unable to seek to directory
	#[error("Unable to seek to directory")]
	Seek(#[source] io::Error),
}

/// Error for [`DirPtr::read_entries`](super::DirPtr::read_entries)
#[derive(Debug, thiserror::Error)]
pub enum ReadEntryError {
	/// Unable to read entry bytes
	#[error("Unable to read entry bytes")]
	ReadEntry(#[source] io::Error),

	/// Unable to parse entry
	#[error("Unable to parse entry")]
	ParseEntry(#[source] crate::entry::DeserializeBytesError),
}


/// Error for [`DirPtr::write_entries`](super::DirPtr::write_entries)
#[derive(Debug, thiserror::Error)]
pub enum WriteEntriesError {
	/// Unable to seek to directory
	#[error("Unable to seek to directory")]
	Seek(#[source] io::Error),

	/// Unable to write all directory entries
	#[error("Unable to write directory entries")]
	WriteEntry(#[source] io::Error),
}

/// Error for [`DirEntryPtr::write`](super::DirEntryPtr::write)
#[derive(Debug, thiserror::Error)]
pub enum WriteEntryError {
	/// Unable to seek to directory
	#[error("Unable to seek to directory")]
	Seek(#[source] io::Error),

	/// Unable to write all directory entries
	#[error("Unable to write directory entries")]
	WriteEntry(#[source] io::Error),
}

/// Error type for [`DirPtr::find`](super::DirPtr::find)
#[derive(Debug, thiserror::Error)]
pub enum FindError {
	/// Unable to find entry
	#[error("Unable to find entry")]
	FindEntry(#[source] crate::ptr::FindEntryError),

	/// Cannot go back to parent directory
	#[error("Cannot go back to parent directory")]
	ParentDir,

	/// Expected directory
	#[error("Expected directory at {path}")]
	ExpectedDir {
		/// The path that wasn't a directory
		path: PathBuf,
	},

	/// Path was empty
	#[error("Path was empty")]
	EmptyPath,
}

/// Error for [`DirPtr::find_entry`](super::DirPtr::find_entry)
#[derive(Debug, thiserror::Error)]
pub enum FindEntryError {
	/// Unable to seek to directory
	#[error("Unable to seek to directory")]
	SeekDir(#[source] ReadEntriesError),

	/// Unable to read entry
	#[error("Unable to read entry")]
	ReadEntry(#[source] ReadEntryError),

	/// Unable to find entry
	#[error("Unable to find entry")]
	FindEntry,
}
