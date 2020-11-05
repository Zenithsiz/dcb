//! Errors

// Imports
use crate::game::card::property::card_type;

/// Error type for [`Bytes::from_bytes`](dcb_bytes::Bytes::from_bytes)
#[derive(PartialEq, Eq, Clone, Copy, Debug, thiserror::Error)]
pub enum FromBytesError {
	/// Unable to parse card type
	#[error("Unable to parse the card type")]
	CardType(#[source] card_type::FromBytesError),
}
