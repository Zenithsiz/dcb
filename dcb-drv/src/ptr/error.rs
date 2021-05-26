//! Errors

// Imports
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
	ParseEntry(#[source] crate::entry::FromBytesError),
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
