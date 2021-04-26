#![doc(include = "table.md")]

// Modules
pub mod error;
pub mod header;

// Exports
pub use error::{DeserializeError, SerializeError};
pub use header::Header;

// Imports
use super::CardHeader;
use crate::card::{self, property::CardType, Digimon, Digivolve, Item};
use dcb_bytes::Bytes;
use std::{convert::TryInto, io};

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
	/// The file of the card table
	pub const PATH: &'static str = "B:\\CARD2.CDD";
}

// Utils
impl Table {
	/// Returns how many cards are in this table
	#[must_use]
	pub fn card_count(&self) -> usize {
		self.digimons.len() + self.items.len() + self.digivolves.len()
	}
}

impl Table {
	/// Deserializes the card table from it's file
	pub fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, DeserializeError> {
		// Read header
		let mut header_bytes = <Header as Bytes>::ByteArray::default();
		reader
			.read_exact(&mut header_bytes)
			.map_err(DeserializeError::ReadHeader)?;
		let header = Header::from_bytes(&header_bytes).map_err(DeserializeError::ParseHeader)?;

		// Then check the number of each card
		let digimon_cards: usize = header.digimons_len.into();
		let item_cards: usize = header.items_len.into();
		let digivolve_cards: usize = header.digivolves_len.into();
		log::trace!("Found {digimon_cards} digimon, {item_cards} item, {digivolve_cards} digivolve cards");

		// And calculate the number of cards
		let cards_len = digimon_cards + item_cards + digivolve_cards;

		// Create the arrays with capacity
		let mut digimons = Vec::with_capacity(digimon_cards);
		let mut items = Vec::with_capacity(item_cards);
		let mut digivolves = Vec::with_capacity(digivolve_cards);

		// Read until the table is over
		for card_id in 0..cards_len {
			// Read card header bytes
			let mut card_header_bytes = [0u8; 0x3];
			reader
				.read_exact(&mut card_header_bytes)
				.map_err(|err| DeserializeError::ReadCardHeader { id: card_id, err })?;

			// Read the header
			let card_header = CardHeader::from_bytes(&card_header_bytes)
				.map_err(|err| DeserializeError::ParseCardHeader { id: card_id, err })?;

			log::trace!("Found #{}: {}", card_id, card_header.ty);

			// If the card id isn't what we expected, log warning
			if usize::from(card_header.id) != card_id {
				log::warn!("Card with id {} had unexpected id {}", card_id, card_header.id);
			}
			// And create / push the card
			match card_header.ty {
				CardType::Digimon => {
					let mut digimon_bytes = [0; std::mem::size_of::<<Digimon as Bytes>::ByteArray>()];
					reader
						.read_exact(&mut digimon_bytes)
						.map_err(|err| DeserializeError::ReadCard {
							id: card_id,
							card_type: card_header.ty,
							err,
						})?;
					let digimon = Digimon::from_bytes(&digimon_bytes)
						.map_err(|err| DeserializeError::ParseDigimonCard { id: card_id, err })?;
					digimons.push(digimon);
				},
				CardType::Item => {
					let mut item_bytes = [0; std::mem::size_of::<<Item as Bytes>::ByteArray>()];
					reader
						.read_exact(&mut item_bytes)
						.map_err(|err| DeserializeError::ReadCard {
							id: card_id,
							card_type: card_header.ty,
							err,
						})?;
					let item = Item::from_bytes(&item_bytes)
						.map_err(|err| DeserializeError::ParseItemCard { id: card_id, err })?;
					items.push(item);
				},
				CardType::Digivolve => {
					let mut digivolve_bytes = [0; std::mem::size_of::<<Digivolve as Bytes>::ByteArray>()];
					reader
						.read_exact(&mut digivolve_bytes)
						.map_err(|err| DeserializeError::ReadCard {
							id: card_id,
							card_type: card_header.ty,
							err,
						})?;
					let digivolve = Digivolve::from_bytes(&digivolve_bytes)
						.map_err(|err| DeserializeError::ParseDigivolveCard { id: card_id, err })?;
					digivolves.push(digivolve);
				},
			}

			// Skip null terminator
			let mut null_terminator = [0; 1];
			reader
				.read_exact(&mut null_terminator)
				.map_err(|err| DeserializeError::ReadCardFooter { id: card_id, err })?;
			if null_terminator[0] != 0 {
				log::warn!(
					"Card with id {}'s null terminator was {} instead of 0",
					card_id,
					null_terminator[0]
				);
			}
		}

		// Return the table
		Ok(Self {
			digimons,
			items,
			digivolves,
		})
	}

	/// Serializes this card table to a file
	pub fn serialize<R: io::Write>(&self, mut writer: R) -> Result<(), SerializeError> {
		// Write header
		let header = Header {
			digimons_len:   self
				.digimons
				.len()
				.try_into()
				.map_err(|_err| SerializeError::TooManyDigimon(self.digimons.len()))?,
			items_len:      self
				.items
				.len()
				.try_into()
				.map_err(|_err| SerializeError::TooManyItems(self.items.len()))?,
			digivolves_len: self
				.digivolves
				.len()
				.try_into()
				.map_err(|_err| SerializeError::TooManyDigivolves(self.digivolves.len()))?,
		};
		let mut header_bytes = [0u8; 0x8];
		header.to_bytes(&mut header_bytes).into_ok();
		writer.write_all(&header_bytes).map_err(SerializeError::WriteHeader)?;

		// Macro to help write all cards to file
		macro_rules! write_card {
			($cards:expr, $prev_ids:expr, $card_type:ident, $($on_err:tt)*) => {{
				for (rel_id, card) in $cards.iter().enumerate() {
					// Current id through the whole table
					let cur_id = $prev_ids + rel_id;

					// Card bytes
					let mut card_bytes = [0; 0x3 + CardType::$card_type.byte_size() + 0x1];
					let bytes = dcb_util::array_split_mut!(&mut card_bytes,
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
					#[allow(unreachable_code)] // Might be `!`
					card
						.to_bytes(bytes.card)
						.map_err(|err| {$($on_err)*}(err, cur_id))?;

					// Write the footer
					*bytes.footer = 0;

					log::trace!("#{}: Writing {}", cur_id, CardType::$card_type);
					writer.write_all(&card_bytes)
						.map_err(|err| SerializeError::WriteCard {
							id: cur_id,
							card_type: CardType::$card_type,
							err
						})?;
				}
			}}
		}

		// Write all cards
		write_card! { self.digimons, 0, Digimon,
			|err, cur_id| SerializeError::SerializeDigimonCard { id: cur_id, err }
		}
		write_card! { self.items, self.digimons.len(), Item,
			|err, cur_id| SerializeError::SerializeItemCard { id: cur_id, err }
		}
		write_card! { self.digivolves, self.digimons.len() + self.items.len(), Digivolve,
			|err, _| err
		}

		// And return Ok
		Ok(())
	}
}
