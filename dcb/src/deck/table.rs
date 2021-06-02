//! The table of all decks in the game

// Modules
pub mod error;

// Exports
pub use error::{DeserializeError, SerializeError};

// Imports
use crate::Deck;
use byteorder::{ByteOrder, LittleEndian};
use dcb_bytes::Bytes;
use dcb_util::array_split_mut;
use std::{convert::TryInto, io};

/// The decks table, where all decks are stored
#[derive(PartialEq, Eq, Clone, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::unsafe_derive_deserialize)] // False positive
pub struct Table {
	/// All decks
	pub decks: Vec<Deck>,
}

// Constants
impl Table {
	/// Table header size
	pub const HEADER_BYTE_SIZE: usize = 0x8;
	/// The magic in the table header
	/// = "33KD"
	pub const HEADER_MAGIC: u32 = 0x444b3033;
}

impl Table {
	/// Deserializes the deck table from `file`.
	pub fn deserialize<R: io::Read>(file: &mut R) -> Result<Self, DeserializeError> {
		// Read header
		let mut header_bytes = [0u8; Self::HEADER_BYTE_SIZE];
		file.read_exact(&mut header_bytes)
			.map_err(DeserializeError::ReadHeader)?;

		// Check if the magic is right
		let magic = LittleEndian::read_u32(&header_bytes[0x0..0x4]);
		if magic != Self::HEADER_MAGIC {
			return Err(DeserializeError::HeaderMagic { magic });
		}

		// Extract the number of decks
		let decks_count: usize = header_bytes[0x4].into();
		log::trace!("Found {decks_count} decks");

		// Then get each deck
		let mut decks = vec![];
		for id in 0..decks_count {
			// Read all bytes of the deck
			let mut bytes = [0; 0x6e];
			file.read_exact(&mut bytes)
				.map_err(|err| DeserializeError::ReadDeck { id, err })?;

			// And try to serialize the deck
			let deck = Deck::deserialize_bytes(&bytes).map_err(|err| DeserializeError::DeserializeDeck { id, err })?;

			// Log the deck
			log::trace!("Found deck #{}: {}", id, deck.name);

			// And add it
			decks.push(deck);
		}

		// And return the table
		Ok(Self { decks })
	}

	/// Serializes the deck table to `file`
	pub fn serialize<R: io::Write>(&self, file: &mut R) -> Result<(), SerializeError> {
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
			deck.serialize_bytes(&mut bytes).into_ok();

			// And write them to file
			file.write(&bytes)
				.map_err(|err| SerializeError::WriteDeck { id, err })?;
		}

		// And return Ok
		Ok(())
	}
}
