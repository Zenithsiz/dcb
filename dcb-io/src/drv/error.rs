//! Errors

// Imports
use super::dir;

/// Error for [`DrvFsReader::from_reader`](super::DrvFsReader::from_reader)
#[derive(Debug, thiserror::Error)]
pub enum FromReaderError {
	/// Unable to read root directory
	#[error("Unable to read root directory")]
	RootDir(#[source] dir::reader::ReadEntryError),
}

/// Error for [`DrvFsWriter::to_writer`](super::DrvFsWriter::to_writer)
#[derive(Debug, thiserror::Error)]
pub enum ToWriterError<E: std::error::Error + 'static> {
	/// Unable to write root directory
	#[error("Unable to write root directory")]
	RootDir(#[source] dir::writer::WriteEntriesError<E>),
}
