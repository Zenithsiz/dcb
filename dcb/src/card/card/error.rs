//! Errors

// Imports
use crate::card;

/// Error type for [`Table::deserialize`](super::Table::deserialize)
#[derive(Debug, thiserror::Error)]
pub enum DeserializeError {
	/// Unable to read card
	#[error("Unable to read card")]
	Read(#[source] std::io::Error),

	/// Unable to deserialize a digimon card
	#[error("Unable to deserialize digimon card")]
	ParseDigimon(#[source] card::digimon::FromBytesError),

	/// Unable to deserialize an item card
	#[error("Unable to deserialize item card")]
	ParseItem(#[source] card::item::FromBytesError),

	/// Unable to deserialize a digivolve card
	#[error("Unable to deserialize digivolve card")]
	ParseDigivolve(#[source] card::digivolve::FromBytesError),
}

/// Error type for [`Table::serialize`](super::Table::serialize)
#[derive(Debug, thiserror::Error)]
pub enum SerializeError {
	/// Unable to write a card
	#[error("Unable to write card")]
	Write(#[source] std::io::Error),

	/// Unable to serialize a digimon card
	#[error("Unable to serialize digimon card")]
	SerializeDigimon(#[source] card::digimon::ToBytesError),

	/// Unable to serialize an item card
	#[error("Unable to serialize item card")]
	SerializeItem(#[source] card::item::ToBytesError),
}
