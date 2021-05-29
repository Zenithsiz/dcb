//! Errors

// Imports
use crate::tim;
use std::io;

/// Error type for [`Tis::deserialize`](super::Tis::deserialize)
#[derive(Debug, thiserror::Error)]
pub enum DeserializeError {
	/// Unable to read magic
	#[error("Unable to read magic")]
	ReadMagic(#[source] io::Error),

	/// Invalid magic
	#[error("Invalid magic {_0:#x}")]
	InvalidMagic(u16),

	/// Unable to read entries len
	#[error("Unable to read entries len")]
	ReadEntriesLen(#[source] io::Error),

	/// Unable to read entries
	#[error("Unable to read entries")]
	ReadEntries(#[source] io::Error),

	/// Unable to seek to tim
	#[error("Unable to seek to tim")]
	SeekTim(#[source] io::Error),

	/// Unable to deserialize tim
	#[error("Unable to deserialize tim")]
	DeserializeTim(#[source] tim::DeserializeError),
}
