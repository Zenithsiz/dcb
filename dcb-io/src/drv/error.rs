//! Errors

// Imports
use super::dir;

/// Error for [`DrvFsReader::from_reader`](super::DrvFsReader::from_reader)
#[derive(Debug, thiserror::Error)]
pub enum FromReaderError {
	/// Unable to read root directory
	#[error("Unable to read root directory")]
	RootDir(#[source] dir::FromReaderError),
}
