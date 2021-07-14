//! Error

// Imports
use std::io;

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
