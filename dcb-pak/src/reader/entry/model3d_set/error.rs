//! Errors

// Imports
use super::model;
use std::io;
use zutil::null_ascii_string;

/// Error for [`Model3dSet::from_reader`](super::Model3dSet::from_reader)
#[derive(Debug, thiserror::Error)]
pub enum FromReaderError {
	/// Unable to read header
	#[error("Unable to read header")]
	ReadHeader(#[source] io::Error),

	/// Unable to parse name
	#[error("Unable to parse name")]
	ParseName(#[source] null_ascii_string::ReadError),

	/// Unable to read unknown1
	#[error("Unable to read unknown1")]
	ReadUnknown1(#[source] io::Error),

	/// Unable to get position
	#[error("Unable to get position")]
	GetPos(#[source] io::Error),

	/// Unable to read model
	#[error("Unable to read model")]
	ReadModel(#[source] model::FromReaderError),
}
