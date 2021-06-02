//! Errors

// Imports
use super::Table;
use crate::deck::deck;
use dcb_bytes::bytes_io_ext::{ReadDeserializeError, WriteSerializeError};

/// Error type for [`Table::deserialize`]
#[derive(Debug, thiserror::Error)]
pub enum DeserializeError {
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

	/// Could not read a deck entry
	#[error("Unable to read deck entry with id {}", id)]
	ReadDeck {
		/// Id of card
		id: usize,

		/// Underlying error
		#[source]
		err: ReadDeserializeError<deck::DeserializeBytesError>,
	},
}

/// Error type for [`Table::serialize`]
#[derive(Debug, thiserror::Error)]
pub enum SerializeError {
	/// Unable to write table header
	#[error("Unable to write table header")]
	WriteHeader(#[source] std::io::Error),

	/// Could not write a deck entry
	#[error("Unable to write deck entry with id {}", id)]
	WriteDeck {
		/// Id of card
		id: usize,

		/// Underlying error
		#[source]
		err: WriteSerializeError<!>,
	},
}
