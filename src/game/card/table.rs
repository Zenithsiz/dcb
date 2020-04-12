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

// Io Traits
use std::io::{Read, Write, Seek};

// byteorder
use byteorder::{ByteOrder, LittleEndian};

// Dcb
use crate::{
	io::{address::Data, GameFile},
	game::{
		card::{
			Digimon, Item, Digivolve,
			property::{CardType, card_type::UnknownCardType},
		},
		Bytes, FromBytes,
	}
};

/// The table storing all cards
#[derive(Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Table {
	pub digimons  : Vec<Digimon  >,
	pub items     : Vec<Item     >,
	pub digivolves: Vec<Digivolve>,
}

// Constants
impl Table {
	/// The start address of the card table
	pub const START_ADDRESS: Data = Data::from_u64(0x216d000);
	
	/// Table header size
	pub const HEADER_BYTE_SIZE: usize = 0x8;
	
	/// The max size of the card table
	// TODO: Check the theoretical max, which is currently thought to be `0x14ff5`
	pub const MAX_BYTE_SIZE: usize = 0x14970;
	
	/// The magic in the table header
	pub const HEADER_MAGIC: u32 = 0x44434130;
}

// Utils
impl Table {
	/// Returns how many cards are in this table
	#[must_use]
	pub fn card_count(&self) -> usize {
		self.digimons  .len() +
		self.items     .len() +
		self.digivolves.len()
	}
}


/// Error type for [`Structure::DeserializeError`]
#[derive(Debug)]
#[derive(derive_more::Display)]
pub enum DeserializeError {
	/// Unable to seek game file
	#[display(fmt = "Unable to seek game file to card table")]
	Seek( std::io::Error ),
	
	/// Unable to read table header
	#[display(fmt = "Unable to read table header")]
	ReadHeader( std::io::Error ),
	
	/// The magic of the table was wrong
	#[display(fmt = "Found wrong table header magic (expected {:x}, found {:x})", Table::HEADER_MAGIC, "magic")]
	HeaderMagic {
		magic: u32,
	},
	
	/// There were too many cards
	#[display(fmt = "Too many cards in table ({} digimon, {} item, {} digivolve, {} / {} bytes max)",
		"digimon_cards",
		"item_cards",
		"digivolve_cards",
		" digimon_cards * (0x3 + Digimon  ::BUF_BYTE_SIZE + 0x1) +
		     item_cards * (0x3 + Item     ::BUF_BYTE_SIZE + 0x1) +
		digivolve_cards * (0x3 + Digivolve::BUF_BYTE_SIZE + 0x1)",
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
		err: std::io::Error,
	},
	
	/// An unknown card type was found
	#[display(fmt = "Unknown card type for card id {}", id)]
	UnknownCardType {
		id: usize,
		err: UnknownCardType,
	},
	
	/// Unable to read card footer
	#[display(fmt = "Unable to read card footer for card id {}", id)]
	ReadCardFooter {
		id: usize,
		err: std::io::Error,
	},
}

impl std::error::Error for DeserializeError {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		match self {
			Self::Seek(err) |
			Self::ReadHeader(err) |
			Self::ReadCardHeader { err, .. } |
			Self::ReadCardFooter { err, .. } => Some(err),
			Self::UnknownCardType { err, .. } => Some(err),
			Self::HeaderMagic { .. } |
			Self::TooManyCards { .. } => None,
		}
	}
}

impl Table {
	/// Deserializes the card table from a game file
	pub fn deserialize<R: Read + Write + Seek>(file: &mut GameFile<R>) -> Result<Self, DeserializeError> {
		// Seek to the table
		file.seek( std::io::SeekFrom::Start( u64::from( Self::START_ADDRESS ) ) )
			.map_err(DeserializeError::Seek)?;
		
		// Read header
		let mut header_bytes = [0u8; 0x8];
		file.read_exact(&mut header_bytes)
			.map_err(DeserializeError::ReadHeader)?;
		
		// Check if the magic is right
		let magic = LittleEndian::read_u32( &header_bytes[0x0..0x4] );
		if magic != Self::HEADER_MAGIC { return Err( DeserializeError::HeaderMagic{ magic } ); }
		
		// Then check the number of each card
		let   digimon_cards = LittleEndian::read_u16( &header_bytes[0x4..0x6] ) as usize;
		let      item_cards = header_bytes[0x6] as usize;
		let digivolve_cards = header_bytes[0x7] as usize;
		
		// And calculate the number of cards
		let cards_len = digimon_cards + item_cards + digivolve_cards;
		
		// If there are too many cards, return Err
		let table_size =  digimon_cards * (0x3 + Digimon  ::BUF_BYTE_SIZE + 0x1) +
		                            item_cards * (0x3 + Item     ::BUF_BYTE_SIZE + 0x1) +
		                       digivolve_cards * (0x3 + Digivolve::BUF_BYTE_SIZE + 0x1);
		if table_size > Self::MAX_BYTE_SIZE { return Err( DeserializeError::TooManyCards {
			  digimon_cards,
			     item_cards,
			digivolve_cards,
		} ); }
		
		
		// Create the arrays with capacity
		let mut digimons     = Vec::with_capacity(digimon_cards);
		let mut items           = Vec::with_capacity(item_cards);
		let mut digivolves = Vec::with_capacity(digivolve_cards);
		
		// Read until the table is over
		for cur_id in 0..cards_len
		{
			// Read card header bytes
			let mut card_header_bytes = [0u8; 0x3];
			file.read_exact(&mut card_header_bytes)
				.map_err(|err| DeserializeError::ReadCardHeader { id: cur_id, err })?;
			
			// Read the header
			let card_id = LittleEndian::read_u16( &card_header_bytes[0x0..0x2] );
			let card_type = CardType::from_bytes( &card_header_bytes[0x2..0x3] )
				.map_err(|err| DeserializeError::UnknownCardType{ id: cur_id, err } )?;
			
			// If the card id isn't what we expected, log warning
			if usize::from(card_id) != cur_id {
				log::warn!("Card with id {} had unexpected id {}", cur_id, card_id);
			}
			// And create / push the card
			match card_type
			{
				CardType::Digimon => {
					let mut digimon_bytes = [0; Digimon::BUF_BYTE_SIZE];
					file.read_exact(&mut digimon_bytes)
						.expect("Unable to read digimon bytes");
					let digimon = Digimon::from_bytes(&digimon_bytes)
						.expect("Unable to parse digimon bytes");
					digimons.push(digimon);
				},
				CardType::Item => {
					let mut item_bytes = [0; Item::BUF_BYTE_SIZE];
					file.read_exact(&mut item_bytes)
						.expect("Unable to read item bytes");
					let item = Item::from_bytes(&item_bytes)
						.expect("Unable to parse item bytes");
					items.push(item);
				},
				CardType::Digivolve => {
					let mut digivolve_bytes = [0; Digivolve::BUF_BYTE_SIZE];
					file.read_exact(&mut digivolve_bytes)
						.expect("Unable to read digivolve bytes");
					let digivolve = Digivolve::from_bytes(&digivolve_bytes)
						.expect("Unable to parse digivolve bytes");
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
		Ok( Self {
			digimons,
			items,
			digivolves,
		})
	}
	
	pub fn serialize<R: Read + Write + Seek>(&self, _file: &mut GameFile<R>) -> Result<(), !> {
		todo!();
		
		/*
		// Get the final table size
		// Note: + 0x4 here is for the `next` section
		let table_size = self.digimons  .len() * (Digimon  ::BUF_BYTE_SIZE + 0x4) +
		                 self.items     .len() * (Item     ::BUF_BYTE_SIZE + 0x4) +
		                 self.digivolves.len() * (Digivolve::BUF_BYTE_SIZE + 0x4);
		let table_size = table_size as u64;
		
		// If the total table size is bigger than the max, return Err
		assert!(table_size > Table::MAX_BYTE_SIZE as u64, "Table is too big");
		/*
		if table_size > Table::MAX_BYTE_SIZE as u64
		{
			return Err( WriteError::TableTooBig{size: table_size, max: Table::MAX_BYTE_SIZE as u64} );
		}
		*/
		
		// The current id
		let mut cur_id = 0u16;
		
		
		// Seek to the beginning of the card table
		game_file.seek( std::io::SeekFrom::Start( u64::from( Table::START_ADDRESS ) + 0x8 ) )
			.expect("Unable to seek to card table");
		
		// Then write all cards, first digimon, then items, then digivolves
		for (idx, digimon) in self.digimons.iter().enumerate()
		{
			// Get the bytes
			let mut bytes = [0u8; Digimon::BUF_BYTE_SIZE as usize];
			digimon.to_bytes(&mut bytes)
				.expect("Unable to get digimon as bytes");
			
			// Write the digimon buffer
			game_file.write_all(&bytes)
				.expect("Unable to write digimon card");
			
			// And write the 'next' section
			let mut buf = [0u8; 0x4];
			
			match idx {
				num if num + 1 == self.digimons.len() => { CardType::Item   .to_bytes( &mut buf[0x3..0x4] )?; }
				_                                     => { CardType::Digimon.to_bytes( &mut buf[0x3..0x4] )?; }
			}
			
			LittleEndian::write_u16( &mut buf[0x1..0x3], cur_id+1);
			
			game_file.write_all(&buf)
				.expect("")
			
			cur_id += 1;
		}
		
		for (idx, item) in self.items.iter().enumerate()
		{
			// Get the bytes
			let mut bytes = [0u8; Item::BUF_BYTE_SIZE as usize];
			item.to_bytes(&mut bytes).map_err(|err| WriteError::ConvertItem{id: cur_id, err})?;
			
			// Write the item buffer
			game_file.write_all(&bytes).map_err(|err| WriteError::WriteItem{id: cur_id, err})?;
			
			// And write the 'next' section
			let mut buf = [0u8; 0x4];
			
			match idx {
				num if num + 1 == self.items.len() => { CardType::Digivolve.to_bytes( &mut buf[0x3..0x4] )?; }
				_                                  => { CardType::Item     .to_bytes( &mut buf[0x3..0x4] )?; }
			}
			
			LittleEndian::write_u16( &mut buf[0x1..0x3], cur_id+1);
			
			game_file.write_all(&buf).map_err(|err| WriteError::NextEntryInfo{ id: cur_id, err })?;
			
			cur_id += 1;
		}
		
		for (idx, digivolve) in self.digivolves.iter().enumerate()
		{
			// Get the bytes
			let mut bytes = [0u8; Digivolve::BUF_BYTE_SIZE as usize];
			digivolve.to_bytes(&mut bytes).map_err(|err| WriteError::ConvertDigivolve{id: cur_id, err})?;
			
			// Write the digimon buffer
			game_file.write_all(&bytes).map_err(|err| WriteError::WriteDigivolve{id: cur_id, err})?;
			
			// And write the 'next' section
			let mut buf = [0u8; 0x4];
			
			match idx {
				num if num + 1 == self.digivolves.len() => { CardType::Digimon  .to_bytes( &mut buf[0x3..0x4] )?; LittleEndian::write_u16( &mut buf[0x1..0x3], 0       ); }
				_                                       => { CardType::Digivolve.to_bytes( &mut buf[0x3..0x4] )?; LittleEndian::write_u16( &mut buf[0x1..0x3], cur_id+1); }
			}
			
			game_file.write_all(&buf).map_err(|err| WriteError::NextEntryInfo{ id: cur_id, err })?;
			
			cur_id += 1;
		}
		
		
		// And return Ok
		Ok(())
		*/
	}
}
