//! Errors

// Imports
use crate::card::{digimon, digivolve, item};
use dcb_bytes::bytes_io_ext::{ReadDeserializeError, WriteSerializeError};

/// Error type for [`Card::deserialize`](super::Card::deserialize)
#[derive(Debug, thiserror::Error)]
pub enum DeserializeError {
	/// Unable to read a digimon card
	#[error("Unable to read digimon card")]
	Digimon(#[from] ReadDeserializeError<digimon::DeserializeBytesError>),

	/// Unable to read an item card
	#[error("Unable to read item card")]
	Item(#[from] ReadDeserializeError<item::DeserializeBytesError>),

	/// Unable to read a digivolve card
	#[error("Unable to read digivolve card")]
	Digivolve(#[from] ReadDeserializeError<digivolve::DeserializeBytesError>),
}

/// Error type for [`Card::serialize`](super::Card::serialize)
#[derive(Debug, thiserror::Error)]
pub enum SerializeError {
	/// Unable to write a digimon card
	#[error("Unable to write digimon card")]
	Digimon(#[from] WriteSerializeError<digimon::SerializeBytesError>),

	/// Unable to write an item card
	#[error("Unable to write item card")]
	Item(#[from] WriteSerializeError<item::SerializeBytesError>),

	/// Unable to write a digivolve card
	#[error("Unable to read digivolve card")]
	Digivolve(#[from] WriteSerializeError<!>),
}
