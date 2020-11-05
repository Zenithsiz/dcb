//! Errors

// Imports
use super::{card, header, CardType, Table};

/// Error type for [`Table::deserialize`]
#[derive(Debug, thiserror::Error)]
pub enum DeserializeError {
	/// Unable to seek game file
	#[error("Unable to seek game file to card table")]
	Seek(#[source] std::io::Error),

	/// Unable to read table header
	#[error("Unable to read table header")]
	ReadHeader(#[source] std::io::Error),

	/// Unable to parse table header
	#[error("Unable to parse table header")]
	Header(#[source] header::FromBytesError),

	/// There were too many cards
	#[error(
		"Too many cards in table ({digimon_cards} digimon, {item_cards} item, {digivolve_cards} digivolve, {} / {} bytes max)",
		  digimon_cards * (0x3 + CardType::Digimon  .byte_size() + 0x1) +
		     item_cards * (0x3 + CardType::Item     .byte_size() + 0x1) +
		digivolve_cards * (0x3 + CardType::Digivolve.byte_size() + 0x1),
		Table::MAX_BYTE_SIZE
	)]
	TooManyCards {
		/// Number of digimon cards
		digimon_cards: usize,

		/// Number of item cards
		item_cards: usize,

		/// Number of digivolve cards
		digivolve_cards: usize,
	},

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
	DigimonCard {
		/// Id of card
		id: usize,

		/// Underlying error
		#[source]
		err: card::digimon::FromBytesError,
	},

	/// Unable to deserialize an item card
	#[error("Unable to deserialize item card with id {}", id)]
	ItemCard {
		/// Id of card
		id: usize,

		/// Underlying error
		#[source]
		err: card::item::FromBytesError,
	},

	/// Unable to deserialize a digivolve card
	#[error("Unable to deserialize digivolve card with id {}", id)]
	DigivolveCard {
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

/// Error type for [`Table::serialize`]
#[derive(Debug, thiserror::Error)]
pub enum SerializeError {
	/// There were too many cards
	#[error(
		"Too many cards in table ({digimon_cards} digimon, {item_cards} item, {digivolve_cards} digivolve, {} / {} bytes max)",
		  digimon_cards * (0x3 + CardType::Digimon  .byte_size() + 0x1) +
		     item_cards * (0x3 + CardType::Item     .byte_size() + 0x1) +
		digivolve_cards * (0x3 + CardType::Digivolve.byte_size() + 0x1),
		Table::MAX_BYTE_SIZE
	)]
	TooManyCards {
		/// Number of digimon cards
		digimon_cards: usize,

		/// Number of item cards
		item_cards: usize,

		/// Number of digivolve cards
		digivolve_cards: usize,
	},

	/// Unable to seek game file
	#[error("Unable to seek game file to card table")]
	Seek(#[source] std::io::Error),

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
