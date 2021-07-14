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
