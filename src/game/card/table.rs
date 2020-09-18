#![doc(include = "table.md")]

// Modules
pub mod error;

// Exports
pub use error::{DeserializeError, SerializeError};

// Imports
use crate::{
	game::{
		card::{
			self,
			property::{self, CardType},
			Digimon, Digivolve, Item,
		},
		Bytes,
	},
	io::{address::Data, GameFile},
	util::{array_split, array_split_mut},
};
use byteorder::{ByteOrder, LittleEndian};
use std::{
	convert::TryInto,
	io::{Read, Seek, Write},
};

/// The table storing all cards
///
/// See the [module containing](self) this struct for more details
/// on where the table is deserialized from and it's features / restrictions.
#[derive(PartialEq, Eq, Clone, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::unsafe_derive_deserialize)] // We don't have any `unsafe` methods
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
	/// Table header size
	pub const HEADER_BYTE_SIZE: usize = 0x8;
	/// The magic in the table header
	/// = "0ACD"
	pub const HEADER_MAGIC: u32 = 0x44434130;
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

#[allow(clippy::use_self)] // TODO: Remove once `min_const_generics` allows us to use it
impl Table {
	/// Deserializes the card table from a game file
	pub fn deserialize<R: Read + Write + Seek>(file: &mut GameFile<R>) -> Result<Self, DeserializeError> {
		// Seek to the table
		file.seek(std::io::SeekFrom::Start(Self::START_ADDRESS.as_u64()))
			.map_err(DeserializeError::Seek)?;

		// Read header
		let mut header_bytes = [0u8; Table::HEADER_BYTE_SIZE];
		file.read_exact(&mut header_bytes).map_err(DeserializeError::ReadHeader)?;
		let header = array_split! {&header_bytes,
			magic: [0x4],

			digimons_len: [0x2],
			items_len: 1,
			digivolves_len: 1,
		};

		// Check if the magic is right
		let magic = LittleEndian::read_u32(header.magic);
		if magic != Self::HEADER_MAGIC {
			return Err(DeserializeError::HeaderMagic { magic });
		}

		// Then check the number of each card
		let digimon_cards: usize = LittleEndian::read_u16(header.digimons_len).into();
		let item_cards: usize = (*header.items_len).into();
		let digivolve_cards: usize = (*header.digivolves_len).into();
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
		for cur_id in 0..cards_len {
			// Read card header bytes
			let mut card_header_bytes = [0u8; 0x3];
			file.read_exact(&mut card_header_bytes)
				.map_err(|err| DeserializeError::ReadCardHeader { id: cur_id, err })?;

			// Read the header
			let card_id = LittleEndian::read_u16(&card_header_bytes[0x0..0x2]);
			let card_type = CardType::from_bytes(&card_header_bytes[0x2]).map_err(|err| DeserializeError::UnknownCardType { id: cur_id, err })?;

			log::trace!("Found #{}: {}", card_id, card_type);

			// If the card id isn't what we expected, log warning
			if usize::from(card_id) != cur_id {
				log::warn!("Card with id {} had unexpected id {}", cur_id, card_id);
			}
			// And create / push the card
			match card_type {
				CardType::Digimon => {
					let mut digimon_bytes = [0; std::mem::size_of::<<Digimon as Bytes>::ByteArray>()];
					file.read_exact(&mut digimon_bytes)
						.map_err(|err| DeserializeError::ReadCard { id: cur_id, card_type, err })?;
					let digimon = Digimon::from_bytes(&digimon_bytes).map_err(|err| DeserializeError::DigimonCard { id: cur_id, err })?;
					digimons.push(digimon);
				},
				CardType::Item => {
					let mut item_bytes = [0; std::mem::size_of::<<Item as Bytes>::ByteArray>()];
					file.read_exact(&mut item_bytes)
						.map_err(|err| DeserializeError::ReadCard { id: cur_id, card_type, err })?;
					let item = Item::from_bytes(&item_bytes).map_err(|err| DeserializeError::ItemCard { id: cur_id, err })?;
					items.push(item);
				},
				CardType::Digivolve => {
					let mut digivolve_bytes = [0; std::mem::size_of::<<Digivolve as Bytes>::ByteArray>()];
					file.read_exact(&mut digivolve_bytes)
						.map_err(|err| DeserializeError::ReadCard { id: cur_id, card_type, err })?;
					let digivolve = Digivolve::from_bytes(&digivolve_bytes).map_err(|err| DeserializeError::DigivolveCard { id: cur_id, err })?;
					digivolves.push(digivolve);
				},
			}

			// Skip null terminator
			let mut null_terminator = [0; 1];
			file.read_exact(&mut null_terminator)
				.map_err(|err| DeserializeError::ReadCardFooter { id: cur_id, err })?;
			if null_terminator[0] != 0 {
				log::warn!("Card with id {}'s null terminator was {} instead of 0", cur_id, null_terminator[0]);
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
		let header = array_split_mut!(&mut header_bytes,
			magic: [0x4],

			digimons_len: [0x2],
			items_len: 1,
			digivolves_len: 1,
		);

		// Set magic
		LittleEndian::write_u32(header.magic, Self::HEADER_MAGIC);

		// Write card lens
		log::trace!("Writing {} digimon cards", self.digimons.len());
		log::trace!("Writing {} item cards", self.items.len());
		log::trace!("Writing {} digivolve cards", self.digivolves.len());
		LittleEndian::write_u16(
			header.digimons_len,
			self.digimons.len().try_into().expect("Number of digimon cards exceeded `u16`"),
		);
		*header.items_len = self.items.len().try_into().expect("Number of item cards exceeded `u8`");
		*header.digivolves_len = self.digivolves.len().try_into().expect("Number of digivolve cards exceeded `u8`");

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
						header_id  : [0x2],
						header_type: 1,
						card       : [CardType::$card_type.byte_size()],
						footer     : 1,
					);

					// Write the header
					LittleEndian::write_u16(bytes.header_id, cur_id.try_into().expect("Card ID exceeded `u16`"));
					CardType::$card_type.to_bytes(bytes.header_type)?;

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
