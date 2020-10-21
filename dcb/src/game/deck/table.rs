//! The table of all decks in the game

// Modules
pub mod error;

// Exports
pub use error::{DeserializeError, SerializeError};

// Imports
use crate::{
	game::Deck,
	io::{address::Data, GameFile},
	util::array_split_mut,
};
use byteorder::{ByteOrder, LittleEndian};
use dcb_bytes::Bytes;
use std::{
	convert::TryInto,
	io::{Read, Seek, Write},
};

/// The decks table, where all decks are stored
#[derive(PartialEq, Eq, Clone, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::unsafe_derive_deserialize)] // We don't have any `unsafe` methods
pub struct Table {
	/// All decks
	decks: Vec<Deck>,
}

// Constants
impl Table {
	/// Table header size
	pub const HEADER_BYTE_SIZE: usize = 0x8;
	/// The magic in the table header
	/// = "33KD"
	pub const HEADER_MAGIC: u32 = 0x444b3033;
	/// The max size of the deck table
	// TODO: Verify this
	pub const MAX_BYTE_SIZE: usize = 0x4452;
	/// The start address of the decks table
	const START_ADDRESS: Data = Data::from_u64(0x21a6800);
}

impl Table {
	/// Deserializes the deck table from `file`.
	pub fn deserialize<R>(file: &mut GameFile<R>) -> Result<Self, DeserializeError>
	where
		R: Read + Write + Seek,
	{
		// Seek to the beginning of the deck table
		file.seek(std::io::SeekFrom::Start(Self::START_ADDRESS.as_u64()))
			.map_err(DeserializeError::Seek)?;

		// Read header
		let mut header_bytes = [0u8; Self::HEADER_BYTE_SIZE];
		file.read_exact(&mut header_bytes).map_err(DeserializeError::ReadHeader)?;

		// Check if the magic is right
		let magic = LittleEndian::read_u32(&header_bytes[0x0..0x4]);
		if magic != Self::HEADER_MAGIC {
			return Err(DeserializeError::HeaderMagic { magic });
		}

		// Extract the number of decks
		let decks_count: usize = header_bytes[0x4].into();
		log::trace!("Found {decks_count} decks");

		// If there are too many decks, return Err
		if decks_count * std::mem::size_of::<<Deck as Bytes>::ByteArray>() > Self::MAX_BYTE_SIZE {
			return Err(DeserializeError::TooManyDecks { decks_count });
		}

		// Then get each deck
		let mut decks = vec![];
		for id in 0..decks_count {
			// Read all bytes of the deck
			let mut bytes = [0; 0x6e];
			file.read_exact(&mut bytes).map_err(|err| DeserializeError::ReadDeck { id, err })?;

			// And try to serialize the deck
			let deck = Deck::from_bytes(&bytes).map_err(|err| DeserializeError::DeserializeDeck { id, err })?;

			// Log the deck
			log::trace!("Found deck #{}: {}", id, deck.name);

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
		// If the total table size is bigger than the max, return Err
		if self.decks.len() * std::mem::size_of::<<Deck as Bytes>::ByteArray>() > Self::MAX_BYTE_SIZE {
			return Err(SerializeError::TooManyDecks {
				decks_count: self.decks.len(),
			});
		}

		// Seek to the beginning of the deck table
		file.seek(std::io::SeekFrom::Start(Self::START_ADDRESS.as_u64()))
			.map_err(SerializeError::Seek)?;

		// Write header
		let mut header_bytes = [0u8; 0x8];
		let header = array_split_mut!(&mut header_bytes,
			magic: [0x4],

			decks_count: 1,
			_unknown: [0x3],
		);

		// Set magic
		LittleEndian::write_u32(header.magic, Self::HEADER_MAGIC);

		// Write deck len
		log::trace!("Writing {} decks", self.decks.len());
		*header.decks_count = self.decks.len().try_into().expect("Number of decks exceeded `u8`");

		// And write the header
		file.write_all(&header_bytes).map_err(SerializeError::WriteHeader)?;

		// Then write each deck
		for (id, deck) in self.decks.iter().enumerate() {
			// Parse each deck into bytes
			let mut bytes = [0; 0x6e];
			deck.to_bytes(&mut bytes).into_ok();

			// And write them to file
			file.write(&bytes).map_err(|err| SerializeError::WriteDeck { id, err })?;
		}

		// And return Ok
		Ok(())
	}
}
