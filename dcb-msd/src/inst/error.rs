//! Errors

// Imports
use std::{io, num::TryFromIntError};

/// Error for [`Inst::encode`](super::Inst::encode)
#[derive(Debug, thiserror::Error)]
pub enum EncodeError {
	/// Unable to write
	#[error("Unable to write")]
	Write(#[from] io::Error),

	/// Unable to write
	#[error("Unable to convert bytes length to a `u16`")]
	LenToU16(#[source] TryFromIntError),
}
