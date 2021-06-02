#![doc(include = "table.md")]

// Modules
pub mod error;
pub mod header;

// Exports
pub use error::{DeserializeError, SerializeError};
pub use header::Header;

// Imports
use super::{Card, CardHeader};
use crate::card::{self, Digimon, Digivolve, Item};
use dcb_bytes::Bytes;
use std::{convert::TryInto, io};

/// Table storing all cards.
#[derive(PartialEq, Eq, Clone, Hash, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Table {
	/// All cards
	pub cards: Vec<Card>,
}

// Constants
impl Table {
	/// The file of the card table
	pub const PATH: &'static str = "B:\\CARD2.CDD";
}

impl Table {
	/// Returns all digimons
	pub fn digimons(&self) -> impl Iterator<Item = &Digimon> {
		self.cards.iter().filter_map(Card::as_digimon)
	}

	/// Returns all items
	pub fn items(&self) -> impl Iterator<Item = &Item> {
		self.cards.iter().filter_map(Card::as_item)
	}

	/// Returns all digivolve
	pub fn digivolves(&self) -> impl Iterator<Item = &Digivolve> {
		self.cards.iter().filter_map(Card::as_digivolve)
	}

	/// Deserializes the card table from it's file
	pub fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, DeserializeError> {
		// Read header
		let mut header_bytes = <Header as Bytes>::ByteArray::default();
		reader
			.read_exact(&mut header_bytes)
			.map_err(DeserializeError::ReadHeader)?;
		let header = Header::from_bytes(&header_bytes).map_err(DeserializeError::ParseHeader)?;

		// Then check the number of each card
		let digimon_cards = header.digimons_len;
		let item_cards = u16::from(header.items_len);
		let digivolve_cards = u16::from(header.digivolves_len);
		let cards_len = digimon_cards + item_cards + digivolve_cards;
		log::trace!(
			"Found {cards_len} cards: {digimon_cards} digimons, {item_cards} items, {digivolve_cards} digivolves"
		);

		// Create the cards
		let cards = (0..cards_len)
			.map(|id| {
				// Read card header bytes
				let mut card_header_bytes = [0u8; 0x3];
				reader
					.read_exact(&mut card_header_bytes)
					.map_err(|err| DeserializeError::ReadCardHeader { id, err })?;

				// Read the header
				let card_header = CardHeader::from_bytes(&card_header_bytes)
					.map_err(|err| DeserializeError::ParseCardHeader { id, err })?;
				log::trace!("#{}: {}", id, card_header.ty);

				// If the card id isn't what we expected, log warning
				if card_header.id != id {
					log::warn!("Card with id {} had unexpected id {}", id, card_header.id);
				}
				// And create the card
				let card = Card::deserialize(card_header.ty, reader)
					.map_err(|err| DeserializeError::DeserializeCard { id, err })?;

				// Skip null terminator
				let mut terminator = 0;
				reader
					.read_exact(std::slice::from_mut(&mut terminator))
					.map_err(|err| DeserializeError::ReadCardFooter { id, err })?;
				if terminator != 0 {
					return Err(DeserializeError::NullTerminator { id, terminator });
				}

				Ok(card)
			})
			.collect::<Result<_, _>>()?;

		// Return the table
		Ok(Self { cards })
	}

	/// Serializes this card table to a file
	pub fn serialize<R: io::Write>(&self, mut writer: R) -> Result<(), SerializeError> {
		// If the cards aren't in order digimon-item-digivolve, return Error
		if !self.cards.iter().is_partitioned(Card::is_digimon) ||
			!self
				.cards
				.iter()
				.is_partitioned(|card| card.is_digimon() || card.is_item())
		{
			return Err(SerializeError::Partitioned);
		}

		// Write header
		let digimons_len = self.digimons().count();
		let items_len = self.items().count();
		let digivolves_len = self.digivolves().count();
		let header = Header {
			digimons_len:   digimons_len
				.try_into()
				.map_err(|_err| SerializeError::TooManyDigimon(digimons_len))?,
			items_len:      items_len
				.try_into()
				.map_err(|_err| SerializeError::TooManyItems(items_len))?,
			digivolves_len: digivolves_len
				.try_into()
				.map_err(|_err| SerializeError::TooManyDigivolves(digivolves_len))?,
		};
		writer
			.write_all(&header.bytes().into_ok())
			.map_err(SerializeError::WriteHeader)?;

		for (card, id) in self.cards.iter().zip(0..) {
			// Write the header
			let header = CardHeader { id, ty: card.ty() };
			writer
				.write_all(&header.bytes().into_ok())
				.map_err(|err| SerializeError::WriteCardHeader { id, err })?;

			// Then serialize the card
			card.serialize(&mut writer)
				.map_err(|err| SerializeError::SerializeCard { id, err })?;

			// And write the footer
			writer
				.write_all(&[0])
				.map_err(|err| SerializeError::WriteCardFooter { id, err })?;
		}

		// And return Ok
		Ok(())
	}
}
