//! Errors

// Imports
use super::dir;

/// Error for [`DrvFsWriter::write_fs`](super::DrvFsWriter::write_fs)
#[derive(Debug, thiserror::Error)]
pub enum WriteFsError<E: std::error::Error + 'static> {
	/// Unable to write root directory
	#[error("Unable to write root directory")]
	RootDir(#[source] dir::writer::WriteEntriesError<E>),
}
