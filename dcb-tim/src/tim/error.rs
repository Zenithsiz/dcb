//! Errors

// Imports
use crate::{clut, header, img};
use std::io;

/// Error type for [`Tim::deserialize`](super::Tim::deserialize)
#[derive(Debug, thiserror::Error)]
pub enum DeserializeError {
	/// Unable to read header
	#[error("Unable to read header")]
	ReadHeader(#[source] io::Error),

	/// Unable to parse header
	#[error("Unable to parse header")]
	ParseHeader(#[source] header::FromBytesError),

	/// Unable to deserialize clut
	#[error("Unable to deserialize clut")]
	DeserializeClut(#[source] clut::DeserializeError),

	/// Unable to deserialize image
	#[error("Unable to deserialize image")]
	DeserializeImg(#[source] img::DeserializeError),
}

/// Error type for [`Tim::colors`](super::Tim::colors)
#[derive(Debug, thiserror::Error)]
pub enum ColorsError {
	/// Missing clut with indexes colors
	#[error("Missing clut with indexed colors")]
	MissingClut,
}
