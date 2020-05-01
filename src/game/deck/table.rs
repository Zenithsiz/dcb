//! The table of all decks in the game

// Std
use std::io::{Read, Seek, Write};

// byteorder
use byteorder::{ByteOrder, LittleEndian};

// Crate
use crate::{
	game::Deck,
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
	/*
	/// The magic of the table was wrong
	#[display(fmt = "Found wrong table header magic (expected {:x}, found {:x})", Table::HEADER_MAGIC, "magic")]
	HeaderMagic { magic: u32 },
	*/
	/// Could not read a deck entry from the deck table
	#[display(fmt = "Unable to fully read a deck entry (The file was too small)")]
	DeckEntry(#[error(source)] std::io::Error),
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

		// Then loop until we're at the end of the table
		//'table_loop: loop
		for _ in 0..100 {
			// Read the deck
			let mut buf = [0u8; 110];
			game_file.read_exact(&mut buf).map_err(DeserializeError::DeckEntry)?;

			// And construct the deck
			let deck = Deck {
				cards: {
					let mut cards_buf = [0u16; 30];

					for card_id in 0..30 {
						cards_buf[card_id] = LittleEndian::read_u16(&buf[0x0 + card_id * 2..0x2 + card_id * 2]);
					}

					cards_buf
				},
			};

			// And add it
			decks.push(deck);
		}

		// And return the table
		Ok(Self { decks })
	}
}
