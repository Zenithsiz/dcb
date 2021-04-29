//! Errors

// Imports
use crate::header;

/// Error type for [`ExeReader::deserialize`](super::ExeReader::deserialize)
#[derive(Debug, thiserror::Error)]
pub enum DeserializeError {
	/// Unable to seek game file
	#[error("Unable to seek game file to executable")]
	Seek(#[source] std::io::Error),

	/// Unable to read header
	#[error("Unable to read header")]
	ReadHeader(#[source] std::io::Error),

	/// Unable to parse header
	#[error("Unable to parse header")]
	ParseHeader(#[source] header::FromBytesError),

	/// Unable to read data
	#[error("Unable to read data")]
	ReadData(#[source] std::io::Error),
}
