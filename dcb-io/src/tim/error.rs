//! Errors

// Imports
use super::clut;
use std::io;

/// Error for [`TimFile::deserialize`](super::TimFile::deserialize)
#[derive(Debug, thiserror::Error)]
pub enum DeserializeError {
	/// Unable to read header
	#[error("Unable to read header")]
	ReadHeader(#[source] io::Error),

	/// Invalid magic
	#[error("Invalid magic {_0:#x}")]
	InvalidMagic(u8),

	/// Invalid version
	#[error("Invalid version {_0:#x}")]
	InvalidVersion(u8),

	/// Unable to read clut
	#[error("Unable to read clut")]
	Clut(#[source] clut::DeserializeError),
}
