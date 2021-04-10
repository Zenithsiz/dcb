//! Errors

// Imports
use super::directive::DecodeWithDataError;
use crate::Data;

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
