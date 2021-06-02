//! Errors

// Imports
use crate::card;

/// Error type for [`Card::deserialize`](super::Card::deserialize)
#[derive(Debug, thiserror::Error)]
pub enum DeserializeError {
	/// Unable to read card
	#[error("Unable to read card")]
	Read(#[source] std::io::Error),

	/// Unable to deserialize a digimon card
	#[error("Unable to deserialize digimon card")]
	ParseDigimon(#[source] card::digimon::DeserializeBytesError),

	/// Unable to deserialize an item card
	#[error("Unable to deserialize item card")]
	ParseItem(#[source] card::item::DeserializeBytesError),

	/// Unable to deserialize a digivolve card
	#[error("Unable to deserialize digivolve card")]
	ParseDigivolve(#[source] card::digivolve::DeserializeBytesError),
}

/// Error type for [`Card::serialize`](super::Card::serialize)
#[derive(Debug, thiserror::Error)]
pub enum SerializeError {
	/// Unable to write a card
	#[error("Unable to write card")]
	Write(#[source] std::io::Error),

	/// Unable to serialize a digimon card
	#[error("Unable to serialize digimon card")]
	SerializeDigimon(#[source] card::digimon::SerializeBytesError),

	/// Unable to serialize an item card
	#[error("Unable to serialize item card")]
	SerializeItem(#[source] card::item::SerializeBytesError),
}
