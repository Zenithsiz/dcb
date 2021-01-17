//! Errors

// Imports
use super::sector;

/// Error type for [`GameFile::sector`](super::GameFile::sector)
#[derive(Debug, thiserror::Error)]
pub enum SectorError {
	/// Unable to seek to sector
	#[error("Unable to seek to sector")]
	Seek(#[source] std::io::Error),

	/// Unable to read sector
	#[error("Unable to read sector")]
	Read(#[source] std::io::Error),

	/// Unable to parse sector
	#[error("Unable to parse sector")]
	Parse(#[source] sector::FromBytesError),
}
