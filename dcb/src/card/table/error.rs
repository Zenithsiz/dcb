//! Errors

// Imports
use super::{card, header, CardType};

/// Error type for [`Table::deserialize`](super::Table::deserialize)
#[derive(Debug, thiserror::Error)]
pub enum DeserializeError {
	/// Unable to read table header
	#[error("Unable to read table header")]
	ReadHeader(#[source] std::io::Error),

	/// Unable to parse table header
	#[error("Unable to parse table header")]
	ParseHeader(#[source] header::FromBytesError),

	/// Unable to read card header
	#[error("Unable to read card header for card id {}", id)]
	ReadCardHeader {
		/// Id of card
		id: usize,

		/// Underlying error
		#[source]
		err: std::io::Error,
	},

	/// Unable to parse a card header
	#[error("Unable to parse a card header for card id {id}")]
	ParseCardHeader {
		/// Id of card
		id: usize,

		/// Underlying error
		#[source]
		err: card::header::FromBytesError,
	},

	/// Unable to read a card
	#[error("Unable to read {} with id {}", card_type, id)]
	ReadCard {
		/// Id of card
		id: usize,

		/// Card type
		card_type: CardType,

		/// Underlying error
		#[source]
		err: std::io::Error,
	},

	/// Unable to deserialize a digimon card
	#[error("Unable to deserialize digimon card with id {}", id)]
	ParseDigimonCard {
		/// Id of card
		id: usize,

		/// Underlying error
		#[source]
		err: card::digimon::FromBytesError,
	},

	/// Unable to deserialize an item card
	#[error("Unable to deserialize item card with id {}", id)]
	ParseItemCard {
		/// Id of card
		id: usize,

		/// Underlying error
		#[source]
		err: card::item::FromBytesError,
	},

	/// Unable to deserialize a digivolve card
	#[error("Unable to deserialize digivolve card with id {}", id)]
	ParseDigivolveCard {
		/// Id of card
		id: usize,

		/// Underlying error
		#[source]
		err: card::digivolve::FromBytesError,
	},

	/// Unable to read card footer
	#[error("Unable to read card footer for card id {}", id)]
	ReadCardFooter {
		/// Id of card
		id: usize,

		/// Underlying error
		#[source]
		err: std::io::Error,
	},
}

/// Error type for [`Table::serialize`](super::Table::serialize)
#[derive(Debug, thiserror::Error)]
pub enum SerializeError {
	/// Number of digimons must fit within a `u16`
	#[error("Number of digimons must fit within a `u16` (was {_0})")]
	TooManyDigimon(usize),

	/// Number of items must fit within a `u8`
	#[error("Number of items must fit within a `u8` (was {_0})")]
	TooManyItems(usize),

	/// Number of digivolves must fit within a `u8`
	#[error("Number of digivolves must fit within a `u8` (was {_0})")]
	TooManyDigivolves(usize),

	/// Unable to write table header
	#[error("Unable to write table header")]
	WriteHeader(#[source] std::io::Error),

	/// Unable to write a card
	#[error("Unable to write {} card with id {}", card_type, id)]
	WriteCard {
		/// Id of card
		id: usize,

		/// Card type
		card_type: CardType,

		/// Underlying error
		#[source]
		err: std::io::Error,
	},

	/// Unable to serialize a digimon card
	#[error("Unable to serialize digimon card with id {}", id)]
	SerializeDigimonCard {
		/// Id of card
		id: usize,

		/// Underlying error
		#[source]
		err: card::digimon::ToBytesError,
	},

	/// Unable to serialize an item card
	#[error("Unable to serialize item card with id {}", id)]
	SerializeItemCard {
		/// Id of card
		id: usize,

		/// Underlying error
		#[source]
		err: card::item::ToBytesError,
	},
}
