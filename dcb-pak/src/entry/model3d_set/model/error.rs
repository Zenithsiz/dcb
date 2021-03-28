//! Errors

// Imports
use std::io;

/// Error for [`TmdModel::from_reader`](super::TmdModel::from_reader)
#[derive(Debug, thiserror::Error)]
pub enum FromReaderError {
	/// Unable to read header
	#[error("Unable to read header")]
	ReadHeader(#[source] io::Error),

	/// Invalid magic
	#[error("Invalid magic {_0:?}")]
	InvalidMagic([u8; 4]),

	/// Unable to read object
	#[error("Unable to read object")]
	ReadObj(#[source] io::Error),

	/// Unable to seek past model
	#[error("Unable to seek past model")]
	SeekPastModel(#[source] io::Error),
}
