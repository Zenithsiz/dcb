//! The table of all decks in the game

// Std
use std::io::{Read, Seek, Write};

// Crate
use crate::{
	game::{deck::deck, Bytes, Deck},
	io::{address::Data, GameFile},
};

/// The decks table, where all decks are stored
///
/// # Details
/// This type serves as an interface to this table, being able to read
/// and write to it, it is the only type able to do so, as each deck
/// type may only be converted to and from bytes.
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
pub struct Table {
	decks: Vec<Deck>,
}

// Constants
impl Table {
	/// The start address of the decks table
	const DECK_TABLE_START_ADDRESS: Data = Data::from_u64(0x21a6808);
}

/// Error type for [`Table::deserialize`]
#[derive(Debug)]
#[derive(derive_more::Display, err_impl::Error)]
pub enum DeserializeError {
	/// Unable to seek game file
	#[display(fmt = "Unable to seek game file to card table")]
	Seek(#[error(source)] std::io::Error),

	/// Unable to read table header
	#[display(fmt = "Unable to read table header")]
	ReadHeader(#[error(source)] std::io::Error),

	/// Could not read a deck entry
	#[display(fmt = "Unable to read deck entry with id {}", "id")]
	ReadDeck {
		id: usize,
		#[error(source)]
		err: std::io::Error,
	},

	/// Could not parse a deck entry
	#[display(fmt = "Unable to parse deck entry with id {}", "id")]
	ParseDeck {
		id: usize,
		#[error(source)]
		err: deck::FromBytesError,
	},
}

impl Table {
	pub fn deserialize<F>(game_file: &mut GameFile<F>) -> Result<Self, DeserializeError>
	where
		F: Read + Write + Seek,
	{
		// The deck array
		let mut decks = vec![];

		// Seek to the beginning of the deck table
		game_file
			.seek(std::io::SeekFrom::Start(u64::from(Self::DECK_TABLE_START_ADDRESS)))
			.map_err(DeserializeError::Seek)?;

		// Then get each deck
		for id in 0..159 {
			// Read all bytes of the deck
			let mut bytes = [0; 0x6e];
			game_file.read_exact(&mut bytes).map_err(|err| DeserializeError::ReadDeck { id, err })?;

			// And try to serialize the deck
			let deck = Deck::from_bytes(&bytes).map_err(|err| DeserializeError::ParseDeck { id, err })?;

			// And add it
			decks.push(deck);
		}

		// And return the table
		Ok(Self { decks })
	}
}
