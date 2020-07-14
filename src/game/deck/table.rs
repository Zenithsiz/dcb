//! The table of all decks in the game

// Imports
use crate::{
	game::{deck::deck, Bytes, Deck},
	io::{address::Data, GameFile},
};
use std::io::{Read, Seek, Write};

/// The decks table, where all decks are stored
///
/// # Details
/// This type serves as an interface to this table, being able to read
/// and write to it, it is the only type able to do so, as each deck
/// type may only be converted to and from bytes.
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
pub struct Table {
	/// All decks
	decks: Vec<Deck>,
}

// Constants
impl Table {
	/// The start address of the decks table
	const DECK_TABLE_START_ADDRESS: Data = Data::from_u64(0x21a6808);
}

/// Error type for [`Table::deserialize`]
#[derive(Debug, thiserror::Error)]
pub enum DeserializeError {
	/// Unable to seek game file
	#[error("Unable to seek game file to card table")]
	Seek(#[source] std::io::Error),

	/// Unable to read table header
	#[error("Unable to read table header")]
	ReadHeader(#[source] std::io::Error),

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

	/// Unable to read table header
	#[error("Unable to read table header")]
	WriteHeader(#[source] std::io::Error),

	/// Could not deserialize a deck entry
	#[error("Unable to deserialize deck entry with id {}", id)]
	SerializeDeck {
		/// Id of card
		id: usize,

		/// Underlying error
		#[source]
		err: deck::ToBytesError,
	},

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

impl Table {
	/// Deserializes the deck table from `file`.
	pub fn deserialize<R>(file: &mut GameFile<R>) -> Result<Self, DeserializeError>
	where
		R: Read + Write + Seek,
	{
		// The deck array
		let mut decks = vec![];

		// Seek to the beginning of the deck table
		file.seek(std::io::SeekFrom::Start(u64::from(Self::DECK_TABLE_START_ADDRESS)))
			.map_err(DeserializeError::Seek)?;

		// Then get each deck
		for id in 0..159 {
			// Read all bytes of the deck
			let mut bytes = [0; 0x6e];
			file.read_exact(&mut bytes).map_err(|err| DeserializeError::ReadDeck { id, err })?;

			// And try to serialize the deck
			let deck = Deck::from_bytes(&bytes).map_err(|err| DeserializeError::DeserializeDeck { id, err })?;

			// And add it
			decks.push(deck);
		}

		// And return the table
		Ok(Self { decks })
	}

	/// Serializes the deck table to `file`
	pub fn serialize<R>(&self, file: &mut GameFile<R>) -> Result<(), SerializeError>
	where
		R: Read + Write + Seek,
	{
		// Seek to the beginning of the deck table
		file.seek(std::io::SeekFrom::Start(u64::from(Self::DECK_TABLE_START_ADDRESS)))
			.map_err(SerializeError::Seek)?;

		// Then get each deck
		for (id, deck) in self.decks.iter().enumerate() {
			// Parse each deck into bytes
			let mut bytes = [0; 0x6e];
			deck.to_bytes(&mut bytes).map_err(|err| SerializeError::SerializeDeck { id, err })?;

			// And write them to file
			file.write(&bytes).map_err(|err| SerializeError::WriteDeck { id, err })?;
		}

		// And return Ok
		Ok(())
	}
}
