//! The table of all digimon in the game
//!
//! # Details
//! At address [0x216d000](Table::START_ADDRESS) of the game file, the card table begins
//! with a small header of `0xb` and then the table itself.
//!
//! # Table Layout
//! The digimon table has a max size of [0x14950](Table::MAX_BYTE_SIZE), but does not
//! necessary use all of this space, but it does follow this layout:
//!
//! | Offset | Size     | Type            | Name                 | Details                                                                 |
//! |--------|----------|-----------------|----------------------|-------------------------------------------------------------------------|
//! | 0x0    | 0x4      | u32             | Magic                | Always contains the string "0ACD" (= [0x44434130](Table::HEADER_MAGIC)) |
//! | 0x4    | 0x2      | u16             | Number of digimon    |                                                                         |
//! | 0x6    | 0x1      | u8              | Number of items      |                                                                         |
//! | 0x7    | 0x1      | u8              | Number of digivolves |                                                                         |
//! | 0x8    | variable | \[`CardEntry`\] | Card Entries         | A contigous array of [Card Entry](#card-entry-layout)                   |
//!
//! # Card Entry Layout
//! Each card entry consists of a header of the card
//!
//! | Offset | Size     | Type                                 | Name            | Details                                      |
//! |--------|----------|--------------------------------------|-----------------|----------------------------------------------|
//! | 0x0    | 0x3      | [`Card Header`](#card-header-layout) | Card Header     | The card's header                            |
//! | 0x3    | variable |                                      | Card            | Either a [Digimon], [Item] or [Digivolve]    |
//! | ...    | 0x1      | u8                                   | Null terminator | A null terminator for the card (must be `0`) |
//!
//! # Card Header Layout
//! The card header determines which type of card this card entry has.
//!
//! | Offset | Size | Type         | Name      | Details                                          |
//! |--------|------|--------------|-----------|--------------------------------------------------|
//! | 0x0    | 0x2  | u16          | Card id   | This card's ID                                   |
//! | 0x2    | 0x1  | [`CardType`] | Card type | The card type ([Digimon], [Item] or [Digivolve]) |

// Std
use std::io::{Read, Seek, Write};

// byteorder
use byteorder::{ByteOrder, LittleEndian};

// Crate
use crate::{
	game::{
		card::{
			self,
			property::{self, CardType},
			Digimon, Digivolve, Item,
		},
		util, Bytes,
	},
	io::{address::Data, GameFile},
};

/// The table storing all cards
#[derive(Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
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

	/// The magic of the table was wrong
	#[display(fmt = "Found wrong table header magic (expected {:x}, found {:x})", Table::HEADER_MAGIC, "magic")]
	HeaderMagic { magic: u32 },

	/// There were too many cards
	#[display(
		fmt = "Too many cards in table ({} digimon, {} item, {} digivolve, {} / {} bytes max)",
		"digimon_cards",
		"item_cards",
		"digivolve_cards",
		" digimon_cards * (0x3 + CardType::Digimon  .byte_size() + 0x1) +
		     item_cards * (0x3 + CardType::Item     .byte_size() + 0x1) +
		digivolve_cards * (0x3 + CardType::Digivolve.byte_size() + 0x1)",
		Table::MAX_BYTE_SIZE
	)]
	TooManyCards {
		digimon_cards: usize,
		item_cards: usize,
		digivolve_cards: usize,
	},

	/// Unable to read card header
	#[display(fmt = "Unable to read card header for card id {}", id)]
	ReadCardHeader {
		id: usize,
		#[error(source)]
		err: std::io::Error,
	},

	/// An unknown card type was found
	#[display(fmt = "Unknown card type for card id {}", id)]
	UnknownCardType {
		id: usize,
		#[error(source)]
		err: property::card_type::FromBytesError,
	},

	/// Unable to read card footer
	#[display(fmt = "Unable to read card footer for card id {}", id)]
	ReadCardFooter {
		id: usize,
		#[error(source)]
		err: std::io::Error,
	},
}

/// Error type for [`Table::serialize`]
#[derive(Debug)]
#[derive(derive_more::Display, err_impl::Error)]
pub enum SerializeError {
	/// Unable to seek game file
	#[display(fmt = "Unable to seek game file to card table")]
	Seek(#[error(source)] std::io::Error),

	/// Unable to write table header
	#[display(fmt = "Unable to write table header")]
	WriteHeader(#[error(source)] std::io::Error),

	/// There were too many cards
	#[display(
		fmt = "Too many cards in table ({} digimon, {} item, {} digivolve, {} / {} bytes max)",
		"digimon_cards",
		"item_cards",
		"digivolve_cards",
		" digimon_cards * (0x3 + CardType::Digimon  .byte_size() + 0x1) +
		     item_cards * (0x3 + CardType::Item     .byte_size() + 0x1) +
		digivolve_cards * (0x3 + CardType::Digivolve.byte_size() + 0x1)",
		Table::MAX_BYTE_SIZE
	)]
	TooManyCards {
		digimon_cards: usize,
		item_cards: usize,
		digivolve_cards: usize,
	},

	/// Unable to write a card
	#[display(fmt = "Unable to write card with id {}", id)]
	WriteCard {
		id: usize,
		#[error(source)]
		err: std::io::Error,
	},

	/// Unable to serialize a digimon card
	#[display(fmt = "Unable to serialize digimon card with id {}", id)]
	DigimonCard {
		id: usize,
		#[error(source)]
		err: card::digimon::ToBytesError,
	},

	/// Unable to write an item card
	#[display(fmt = "Unable to write item card with id {}", id)]
	ItemCard {
		id: usize,
		#[error(source)]
		err: card::item::ToBytesError,
	},

	/// Unable to write a digivolve card
	#[display(fmt = "Unable to write digivolve card with id {}", id)]
	DigivolveCard {
		id: usize,
		#[error(source)]
		err: card::digivolve::ToBytesError,
	},
}

impl Table {
	/// Deserializes the card table from a game file
	pub fn deserialize<R: Read + Write + Seek>(file: &mut GameFile<R>) -> Result<Self, DeserializeError> {
		// Seek to the table
		file.seek(std::io::SeekFrom::Start(u64::from(Self::START_ADDRESS)))
			.map_err(DeserializeError::Seek)?;

		// Read header
		let mut header_bytes = [0u8; 0x8];
		file.read_exact(&mut header_bytes).map_err(DeserializeError::ReadHeader)?;

		// Check if the magic is right
		let magic = LittleEndian::read_u32(&header_bytes[0x0..0x4]);
		if magic != Self::HEADER_MAGIC {
			return Err(DeserializeError::HeaderMagic { magic });
		}

		// Then check the number of each card
		let digimon_cards = LittleEndian::read_u16(&header_bytes[0x4..0x6]) as usize;
		let item_cards = header_bytes[0x6] as usize;
		let digivolve_cards = header_bytes[0x7] as usize;
		log::debug!("[Table Header] Found {} digimon cards", digimon_cards);
		log::debug!("[Table Header] Found {} item cards", item_cards);
		log::debug!("[Table Header] Found {} digivolve cards", digivolve_cards);

		// And calculate the number of cards
		let cards_len = digimon_cards + item_cards + digivolve_cards;

		// If there are too many cards, return Err
		let table_size = digimon_cards * (0x3 + CardType::Digimon.byte_size() + 0x1) +
			item_cards * (0x3 + CardType::Item.byte_size() + 0x1) +
			digivolve_cards * (0x3 + CardType::Digivolve.byte_size() + 0x1);
		log::debug!("[Table Header] {} total bytes of cards", table_size);
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

			log::debug!("[Card Header] Found {} with id {}", card_type, card_id);

			// If the card id isn't what we expected, log warning
			if usize::from(card_id) != cur_id {
				log::warn!("Card with id {} had unexpected id {}", cur_id, card_id);
			}
			// And create / push the card
			match card_type {
				CardType::Digimon => {
					let mut digimon_bytes = [0; std::mem::size_of::<<Digimon as Bytes>::ByteArray>()];
					file.read_exact(&mut digimon_bytes).expect("Unable to read digimon bytes");
					let digimon = Digimon::from_bytes(&digimon_bytes).expect("Unable to parse digimon bytes");
					digimons.push(digimon);
				},
				CardType::Item => {
					let mut item_bytes = [0; std::mem::size_of::<<Item as Bytes>::ByteArray>()];
					file.read_exact(&mut item_bytes).expect("Unable to read item bytes");
					let item = Item::from_bytes(&item_bytes).expect("Unable to parse item bytes");
					items.push(item);
				},
				CardType::Digivolve => {
					let mut digivolve_bytes = [0; std::mem::size_of::<<Digivolve as Bytes>::ByteArray>()];
					file.read_exact(&mut digivolve_bytes).expect("Unable to read digivolve bytes");
					let digivolve = Digivolve::from_bytes(&digivolve_bytes).expect("Unable to parse digivolve bytes");
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

	pub fn serialize<R: Read + Write + Seek>(&self, file: &mut GameFile<R>) -> Result<(), SerializeError> {
		// Get the final table size
		let table_size = self.digimons.len() * (0x3 + CardType::Digimon.byte_size() + 0x1) +
			self.items.len() * (0x3 + CardType::Item.byte_size() + 0x1) +
			self.digivolves.len() * (0x3 + CardType::Digivolve.byte_size() + 0x1);

		// If the total table size is bigger than the max, return Err
		if table_size > Self::MAX_BYTE_SIZE {
			return Err(SerializeError::TooManyCards {
				digimon_cards: self.digimons.len(),
				item_cards: self.items.len(),
				digivolve_cards: self.digivolves.len(),
			});
		}

		// Seek to the beginning of the card table
		file.seek(std::io::SeekFrom::Start(u64::from(Self::START_ADDRESS)))
			.map_err(SerializeError::Seek)?;

		// Write header
		let mut header_bytes = [0u8; 0x8];
		{
			let bytes = util::array_split_mut!(&mut header_bytes,
				magic: [0x4],

				digimons_len: [0x2],
				items_len: 1,
				digivolves_len: 1,
			);

			// Set magic
			LittleEndian::write_u32(bytes.magic, Self::HEADER_MAGIC);

			// Write card lens
			use std::convert::TryInto;
			log::debug!("[Table Header] Writing {} digimon cards", self.digimons.len());
			log::debug!("[Table Header] Writing {} item cards", self.items.len());
			log::debug!("[Table Header] Writing {} digivolve cards", self.digivolves.len());
			LittleEndian::write_u16(bytes.digimons_len, self.digimons.len().try_into().expect("Too many digimons"));
			*bytes.items_len = self.items.len().try_into().expect("Too many items");
			*bytes.digivolves_len = self.digivolves.len().try_into().expect("Too many digivolves");
		}

		file.write_all(&header_bytes).map_err(SerializeError::WriteHeader)?;

		// Write all digimon, items and digivolves
		for (rel_id, digimon) in self.digimons.iter().enumerate() {
			// Current id through the whole table
			let cur_id = rel_id;

			// Card bytes
			let mut card_bytes = [0; 0x3 + CardType::Digimon.byte_size() + 0x1];
			let bytes = util::array_split_mut!(&mut card_bytes,
				header_id  : [0x2],
				header_type: 1,
				digimon    : [CardType::Digimon.byte_size()],
				footer     : 1,
			);

			// Write the header
			LittleEndian::write_u16(bytes.header_id, cur_id as u16);
			CardType::Digimon.to_bytes(bytes.header_type)?;

			// Write the digimon
			digimon
				.to_bytes(bytes.digimon)
				.map_err(|err| SerializeError::DigimonCard { id: cur_id, err })?;

			// Write the footer
			*bytes.footer = 0;

			log::debug!("[Card Header] Writing Digimon with id {}", cur_id);
			file.write_all(&card_bytes).map_err(|err| SerializeError::WriteCard { id: cur_id, err })?;
		}
		for (rel_id, item) in self.items.iter().enumerate() {
			// Current id through the whole table
			let cur_id = self.digimons.len() + rel_id;

			// Card bytes
			let mut card_bytes = [0; 0x3 + CardType::Item.byte_size() + 0x1];
			let bytes = util::array_split_mut!(&mut card_bytes,
				header_id  : [0x2],
				header_type: 1,
				item       : [CardType::Item.byte_size()],
				footer     : 1,
			);

			// Write the header
			LittleEndian::write_u16(bytes.header_id, cur_id as u16);
			CardType::Item.to_bytes(bytes.header_type)?;

			// Write the item
			item.to_bytes(bytes.item).map_err(|err| SerializeError::ItemCard { id: cur_id, err })?;

			// Write the footer
			*bytes.footer = 0;

			log::debug!("[Card Header] Writing Item with id {}", cur_id);
			file.write_all(&card_bytes).map_err(|err| SerializeError::WriteCard { id: cur_id, err })?;
		}
		for (rel_id, digivolve) in self.digivolves.iter().enumerate() {
			// Current id through the whole table
			let cur_id = self.digimons.len() + self.items.len() + rel_id;

			// Card bytes
			let mut card_bytes = [0; 0x3 + CardType::Digivolve.byte_size() + 0x1];
			let bytes = util::array_split_mut!(&mut card_bytes,
				header_id  : [0x2],
				header_type: 1,
				item       : [CardType::Digivolve.byte_size()],
				footer     : 1,
			);

			// Write the header
			LittleEndian::write_u16(bytes.header_id, cur_id as u16);
			CardType::Digivolve.to_bytes(bytes.header_type)?;

			// Write the digivolve
			digivolve
				.to_bytes(bytes.item)
				.map_err(|err| SerializeError::DigivolveCard { id: cur_id, err })?;

			// Write the footer
			*bytes.footer = 0;

			log::debug!("[Card Header] Writing Digivolve with id {}", cur_id);
			file.write_all(&card_bytes).map_err(|err| SerializeError::WriteCard { id: cur_id, err })?;
		}

		// And return Ok
		Ok(())
	}
}
