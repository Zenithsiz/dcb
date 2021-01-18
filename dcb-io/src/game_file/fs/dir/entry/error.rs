//! Errors

// Imports
use crate::game_file::fs::dir;
use std::ops::{Range, RangeFrom};

/// Error for [`Bytes::from_bytes`](super::Bytes::from_bytes)
#[derive(Debug, thiserror::Error)]
pub enum FromBytesError {
	/// Invalid kind
	#[error("Invalid kind {_0:#x}")]
	InvalidKind(u8),

	/// Unable to read name
	#[error("Unable to read name")]
	Name(#[source] dcb_util::ascii_str_arr::FromBytesError<0x10>),

	/// Unable to read extension
	#[error("Unable to read extension")]
	Extension(#[source] dcb_util::ascii_str_arr::FromBytesError<0x3>),

	/// Unable to get file contents
	#[error("Unable to get file contents at {_0:?}")]
	ContentsFile(Range<usize>),

	/// Unable to get directory range
	#[error("Unable to get directory range at {_0:?}")]
	ContentsDir(RangeFrom<usize>),

	/// Unable to get directory
	#[error("Unable to read directory")]
	ParseDir(#[source] Box<dir::FromBytesError>),
}
