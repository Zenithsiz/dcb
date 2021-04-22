//! Errors

// Imports
use super::directive::DecodeWithDataError;
use crate::Data;
use ascii::AsAsciiStrError;
use std::num::TryFromIntError;

/// Error type for [`Inst::decode`](super::Inst::decode)
#[derive(Debug, thiserror::Error)]
pub enum DecodeError<'a> {
	/// Invalid data location to read from
	#[error("Attempted to decode instruction from within data location")]
	InvalidDataLocation {
		/// The data location
		data: &'a Data,

		/// Underlying error
		err: DecodeWithDataError,
	},

	/// Bytes is empty
	#[error("No more bytes")]
	NoBytes,
}

/// Error type for [`Inst::from_parsed`](super::Inst::from_parsed)
#[derive(Debug, thiserror::Error)]
pub enum FromParsedError {
	/// Unknown mnemonic
	#[error("Unknown mnemonic")]
	UnknownMnemonic,

	/// Invalid arguments
	#[error("Invalid arguments")]
	InvalidArguments,

	/// Literal is out of range
	#[error("Literal is out of range")]
	LiteralOutOfRange(#[from] TryFromIntError),

	/// String was non-ascii
	#[error("String is non-ascii")]
	StringNonAscii(#[from] AsAsciiStrError),

	/// Unknown label
	#[error("Unknown label {_0:?}")]
	UnknownLabel(String),

	/// Relative jump is too far away
	#[error("Relative jump is too far away")]
	RelativeJumpTooFar(#[source] TryFromIntError),
}
