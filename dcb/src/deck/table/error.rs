//! Errors

// Imports
use super::{Bytes, Deck, Table};
use crate::deck::deck;

/// Error type for [`Table::deserialize`]
#[derive(Debug, thiserror::Error)]
pub enum DeserializeError {
	/// Unable to seek game file
	#[error("Unable to seek game file to card table")]
	Seek(#[source] std::io::Error),

	/// Unable to read table header
	#[error("Unable to read table header")]
	ReadHeader(#[source] std::io::Error),

	/// The magic of the table was wrong
	#[error(
		"Found wrong table header magic (expected {:#}, found {:#x})",
		Table::HEADER_MAGIC,
		magic
	)]
	HeaderMagic {
		/// Magic we found
		magic: u32,
	},

	/// There were too many decks
	#[error(
		"Too many decks in table ({decks_count} decks, {} / 0 bytes max)",
		decks_count * std::mem::size_of::<<Deck as Bytes>::ByteArray>(),
	)]
	TooManyDecks {
		/// Number of decks
		decks_count: usize,
	},

	/// Could not read a deck entry
	#[error("Unable to read deck entry with id {}", id)]
	ReadDeck {
		/// Id of card
		id: usize,

		/// Underlying error
		#[source]
		err: std::io::Error,
	},

	/// Could not deserialize a deck entry
	#[error("Unable to serialize deck entry with id {}", id)]
	DeserializeDeck {
		/// Id of card
		id: usize,

		/// Underlying error
		#[source]
		err: deck::FromBytesError,
	},
}

/// Error type for [`Table::serialize`]
#[derive(Debug, thiserror::Error)]
pub enum SerializeError {
	/// Unable to seek game file
	#[error("Unable to seek game file to card table")]
	Seek(#[source] std::io::Error),

	/// There were too many decks
	#[error(
		"Too many decks in table ({decks_count} decks, {} / 0 bytes max)",
		decks_count * std::mem::size_of::<<Deck as Bytes>::ByteArray>(),
	)]
	TooManyDecks {
		/// Number of decks
		decks_count: usize,
	},

	/// Unable to write table header
	#[error("Unable to write table header")]
	WriteHeader(#[source] std::io::Error),

	/// Could not write a deck entry
	#[error("Unable to read deck entry with id {}", id)]
	WriteDeck {
		/// Id of card
		id: usize,

		/// Underlying error
		#[source]
		err: std::io::Error,
	},
}
