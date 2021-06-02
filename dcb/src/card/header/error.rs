//! Errors

// Imports
use crate::card::property::card_type;

/// Error type for [`Bytes::deserialize_bytes`](dcb_bytes::Bytes::deserialize_bytes)
#[derive(PartialEq, Eq, Clone, Copy, Debug, thiserror::Error)]
pub enum DeserializeBytesError {
	/// Unable to parse card type
	#[error("Unable to parse the card type")]
	CardType(#[source] card_type::DeserializeBytesError),
}
