//! Errors

// Imports
use super::{card, header};

/// Error type for [`Table::deserialize`](super::Table::deserialize)
#[derive(Debug, thiserror::Error)]
pub enum DeserializeError {
	/// Unable to read table header
	#[error("Unable to read table header")]
	ReadHeader(#[source] std::io::Error),

	/// Unable to parse table header
	#[error("Unable to parse table header")]
	ParseHeader(#[source] header::DeserializeBytesError),

	/// Unable to read card header
	#[error("Unable to read card header for card {id}")]
	ReadCardHeader {
		/// Id of card
		id: u16,

		/// Underlying error
		#[source]
		err: std::io::Error,
	},

	/// Unable to parse a card header
	#[error("Unable to parse a card header for card {id}")]
	ParseCardHeader {
		/// Id of card
		id: u16,

		/// Underlying error
		#[source]
		err: card::header::DeserializeBytesError,
	},

	/// Unable to deserialize card
	#[error("Unable to deserialize card {id}")]
	DeserializeCard {
		/// Id of card
		id: u16,

		/// Underlying error
		#[source]
		err: card::card::DeserializeError,
	},

	/// Unable to read card footer
	#[error("Unable to read card footer for card {id}")]
	ReadCardFooter {
		/// Id of card
		id: u16,

		/// Underlying error
		#[source]
		err: std::io::Error,
	},

	/// Null terminator wasn't null
	#[error("Null terminator for card {id} was {terminator}")]
	NullTerminator {
		/// Id of card
		id: u16,

		/// Terminator
		terminator: u8,
	},
}

/// Error type for [`Table::serialize`](super::Table::serialize)
#[derive(Debug, thiserror::Error)]
pub enum SerializeError {
	/// Cards weren't partitioned correctly
	#[error("Cards weren't partitioned correctly")]
	Partitioned,

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

	/// Unable to write card header
	#[error("Unable to write header for card {id}")]
	WriteCardHeader {
		/// Id of card
		id: u16,

		/// Underlying error
		#[source]
		err: std::io::Error,
	},

	/// Unable to serialize card
	#[error("Unable to serialize card {id}")]
	SerializeCard {
		/// Id of card
		id: u16,

		/// Underlying error
		#[source]
		err: card::card::SerializeError,
	},

	/// Unable to write card footer
	#[error("Unable to write footer for card {id}")]
	WriteCardFooter {
		/// Id of card
		id: u16,

		/// Underlying error
		#[source]
		err: std::io::Error,
	},
}
