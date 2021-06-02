//! Errors

// Imports
use crate::card::{digimon, digivolve, item};
use dcb_bytes::bytes_io_ext::{ReadBytesError, WriteBytesError};

/// Error type for [`Card::deserialize`](super::Card::deserialize)
#[derive(Debug, thiserror::Error)]
pub enum DeserializeError {
	/// Unable to read a digimon card
	#[error("Unable to read digimon card")]
	Digimon(#[from] ReadBytesError<digimon::DeserializeBytesError>),

	/// Unable to read an item card
	#[error("Unable to read item card")]
	Item(#[from] ReadBytesError<item::DeserializeBytesError>),

	/// Unable to read a digivolve card
	#[error("Unable to read digivolve card")]
	Digivolve(#[from] ReadBytesError<digivolve::DeserializeBytesError>),
}

/// Error type for [`Card::serialize`](super::Card::serialize)
#[derive(Debug, thiserror::Error)]
pub enum SerializeError {
	/// Unable to write a digimon card
	#[error("Unable to write digimon card")]
	Digimon(#[from] WriteBytesError<digimon::SerializeBytesError>),

	/// Unable to write an item card
	#[error("Unable to write item card")]
	Item(#[from] WriteBytesError<item::SerializeBytesError>),

	/// Unable to write a digivolve card
	#[error("Unable to read digivolve card")]
	Digivolve(#[from] WriteBytesError<!>),
}
