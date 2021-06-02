//! Errors

// Imports
use crate::header;
use dcb_bytes::bytes_io_ext::ReadDeserializeError;

/// Error type for [`ExeReader::deserialize`](super::ExeReader::deserialize)
#[derive(Debug, thiserror::Error)]
pub enum DeserializeError {
	/// Unable to seek game file
	#[error("Unable to seek game file to executable")]
	Seek(#[source] std::io::Error),

	/// Unable to read header
	#[error("Unable to read header")]
	ReadHeader(#[from] ReadDeserializeError<header::DeserializeBytesError>),

	/// Unable to read data
	#[error("Unable to read data")]
	ReadData(#[source] std::io::Error),
}
