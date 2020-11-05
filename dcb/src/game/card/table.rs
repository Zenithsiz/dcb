#![doc(include = "table.md")]

// Modules
pub mod error;
pub mod header;

// Exports
pub use error::{DeserializeError, SerializeError};
pub use header::Header;

// Imports
use crate::{
	game::card::{self, property::CardType, CardHeader, Digimon, Digivolve, Item},
	io::{address::Data, GameFile},
	util::array_split_mut,
};
use dcb_bytes::Bytes;
use std::{
	convert::TryInto,
	io::{Read, Seek, Write},
};

/// Table storing all cards.
#[derive(PartialEq, Eq, Clone, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::unsafe_derive_deserialize)] // False positive
pub struct Table {
	/// All digimons in this table
	pub digimons: Vec<Digimon>,

	/// All items in this table
	pub items: Vec<Item>,

	/// All digivolves in this table
	pub digivolves: Vec<Digivolve>,
}

// Constants
impl Table {
	/// The max size of the card table
	// TODO: Check the theoretical max, which is currently thought to be `0x14ff5`
	pub const MAX_BYTE_SIZE: usize = 0x14970;
	/// The start address of the card table
	pub const START_ADDRESS: Data = Data::from_u64(0x216d000);
}

// Utils
impl Table {
	/// Returns how many cards are in this table
	#[must_use]
	pub fn card_count(&self) -> usize {
		self.digimons.len() + self.items.len() + self.digivolves.len()
	}

	/// Returns the byte size of all cards in this table
	#[must_use]
	#[rustfmt::skip]
	pub fn cards_byte_size(&self) -> usize {
		self.digimons  .len() * (0x3 + CardType::Digimon  .byte_size() + 0x1) +
		self.items     .len() * (0x3 + CardType::Item     .byte_size() + 0x1) +
		self.digivolves.len() * (0x3 + CardType::Digivolve.byte_size() + 0x1)
	}
}

impl Table {
	/// Deserializes the card table from a game file
	pub fn deserialize<R: Read + Write + Seek>(file: &mut GameFile<R>) -> Result<Self, DeserializeError> {
		// Seek to the table
		file.seek(std::io::SeekFrom::Start(Self::START_ADDRESS.as_u64()))
			.map_err(DeserializeError::Seek)?;

		// Read header
		let mut header_bytes = <Header as Bytes>::ByteArray::default();
		file.read_exact(&mut header_bytes).map_err(DeserializeError::ReadHeader)?;
		let header = Header::from_bytes(&header_bytes).map_err(DeserializeError::Header)?;

		// Then check the number of each card
		let digimon_cards: usize = header.digimons_len.into();
		let item_cards: usize = header.items_len.into();
		let digivolve_cards: usize = header.digivolves_len.into();
		log::trace!("Found {digimon_cards} digimon, {item_cards} item, {digivolve_cards} digivolve cards");

		// And calculate the number of cards
		let cards_len = digimon_cards + item_cards + digivolve_cards;

		// If there are too many cards, return Err
		let table_size = digimon_cards * (0x3 + CardType::Digimon.byte_size() + 0x1) +
			item_cards * (0x3 + CardType::Item.byte_size() + 0x1) +
			digivolve_cards * (0x3 + CardType::Digivolve.byte_size() + 0x1);
		if table_size > Self::MAX_BYTE_SIZE {
			return Err(DeserializeError::TooManyCards {
				digimon_cards,
				item_cards,
				digivolve_cards,
			});
		}

		// Create the arrays with capacity
		let mut digimons = Vec::with_capacity(digimon_cards);
		let mut items = Vec::with_capacity(item_cards);
		let mut digivolves = Vec::with_capacity(digivolve_cards);

		// Read until the table is over
		for card_id in 0..cards_len {
			// Read card header bytes
			let mut card_header_bytes = [0u8; 0x3];
			file.read_exact(&mut card_header_bytes)
				.map_err(|err| DeserializeError::ReadCardHeader { id: card_id, err })?;

			// Read the header
			let card_header = CardHeader::from_bytes(&card_header_bytes).map_err(|err| DeserializeError::ParseCardHeader { id: card_id, err })?;

			log::trace!("Found #{}: {}", card_id, card_header.ty);

			// If the card id isn't what we expected, log warning
			if usize::from(card_header.id) != card_id {
				log::warn!("Card with id {} had unexpected id {}", card_id, card_header.id);
			}
			// And create / push the card
			match card_header.ty {
				CardType::Digimon => {
					let mut digimon_bytes = [0; std::mem::size_of::<<Digimon as Bytes>::ByteArray>()];
					file.read_exact(&mut digimon_bytes).map_err(|err| DeserializeError::ReadCard {
						id: card_id,
						card_type: card_header.ty,
						err,
					})?;
					let digimon = Digimon::from_bytes(&digimon_bytes).map_err(|err| DeserializeError::DigimonCard { id: card_id, err })?;
					digimons.push(digimon);
				},
				CardType::Item => {
					let mut item_bytes = [0; std::mem::size_of::<<Item as Bytes>::ByteArray>()];
					file.read_exact(&mut item_bytes).map_err(|err| DeserializeError::ReadCard {
						id: card_id,
						card_type: card_header.ty,
						err,
					})?;
					let item = Item::from_bytes(&item_bytes).map_err(|err| DeserializeError::ItemCard { id: card_id, err })?;
					items.push(item);
				},
				CardType::Digivolve => {
					let mut digivolve_bytes = [0; std::mem::size_of::<<Digivolve as Bytes>::ByteArray>()];
					file.read_exact(&mut digivolve_bytes).map_err(|err| DeserializeError::ReadCard {
						id: card_id,
						card_type: card_header.ty,
						err,
					})?;
					let digivolve = Digivolve::from_bytes(&digivolve_bytes).map_err(|err| DeserializeError::DigivolveCard { id: card_id, err })?;
					digivolves.push(digivolve);
				},
			}

			// Skip null terminator
			let mut null_terminator = [0; 1];
			file.read_exact(&mut null_terminator)
				.map_err(|err| DeserializeError::ReadCardFooter { id: card_id, err })?;
			if null_terminator[0] != 0 {
				log::warn!("Card with id {}'s null terminator was {} instead of 0", card_id, null_terminator[0]);
			}
		}

		// Return the table
		Ok(Self { digimons, items, digivolves })
	}

	/// Serializes this card table to `file`.
	pub fn serialize<R: Read + Write + Seek>(&self, file: &mut GameFile<R>) -> Result<(), SerializeError> {
		// Get the final table size
		let table_size = self.cards_byte_size();

		// If the total table size is bigger than the max, return Err
		if table_size > Self::MAX_BYTE_SIZE {
			return Err(SerializeError::TooManyCards {
				digimon_cards:   self.digimons.len(),
				item_cards:      self.items.len(),
				digivolve_cards: self.digivolves.len(),
			});
		}

		// Seek to the beginning of the card table
		file.seek(std::io::SeekFrom::Start(Self::START_ADDRESS.as_u64()))
			.map_err(SerializeError::Seek)?;

		// Write header
		let mut header_bytes = [0u8; 0x8];
		let header = Header {
			digimons_len:   self.digimons.len().try_into().expect("Number of digimon cards exceeded `u16`"),
			items_len:      self.items.len().try_into().expect("Number of item cards exceeded `u8`"),
			digivolves_len: self.digivolves.len().try_into().expect("Number of digivolve cards exceeded `u8`"),
		};
		header.to_bytes(&mut header_bytes).into_ok();

		// And write the header
		file.write_all(&header_bytes).map_err(SerializeError::WriteHeader)?;

		// Macro to help write all cards to file
		macro_rules! write_card {
			($cards:expr, $prev_ids:expr, $card_type:ident, $($on_err:tt)*) => {
				for (rel_id, card) in $cards.iter().enumerate() {
					// Current id through the whole table
					let cur_id = $prev_ids + rel_id;

					// Card bytes
					let mut card_bytes = [0; 0x3 + CardType::$card_type.byte_size() + 0x1];
					let bytes = array_split_mut!(&mut card_bytes,
						header     : [0x3],
						card       : [CardType::$card_type.byte_size()],
						footer     : 1,
					);

					let card_header = CardHeader {
						id: cur_id.try_into().expect("Card id didn't fit into a `u16`"),
						ty: CardType::$card_type,
					};
					card_header.to_bytes(bytes.header).into_ok();

					// Write the card
					#[allow(unreachable_code)] // FIXME: Remove this
					card
						.to_bytes(bytes.card)
						.map_err(|err| {$($on_err)*}(err, cur_id))?;

					// Write the footer
					*bytes.footer = 0;

					log::trace!("#{}: Writing {}", cur_id, CardType::$card_type);
					file.write_all(&card_bytes)
						.map_err(|err| SerializeError::WriteCard {
							id: cur_id,
							card_type: CardType::$card_type,
							err
						})?;
				}
			}
		}

		// Write all cards
		{
			// Buffer, Offset, Type, Error closure
			write_card! { self.digimons  , 0                                     , Digimon  ,
				|err, cur_id| SerializeError::SerializeDigimonCard { id: cur_id, err }
			}
			write_card! { self.items     , self.digimons.len()                   , Item     ,
				|err, cur_id| SerializeError::SerializeItemCard    { id: cur_id, err }
			}
			write_card! { self.digivolves, self.digimons.len() + self.items.len(), Digivolve,
				|err, _| err
			}
		}

		// And return Ok
		Ok(())
	}
}
