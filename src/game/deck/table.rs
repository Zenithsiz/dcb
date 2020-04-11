// Dcb
//--------------------------------------------------------------------------------------------------
	// Io
	use crate::io::address::Data;
	use crate::io::GameFile;
	
	// Game
	//use crate::game::deck::Deck;
	//use crate::game::Bytes;
	//use crate::game::FromBytes;
	//use crate::game::ToBytes;
//--------------------------------------------------------------------------------------------------

// Read / Write
use std::io::Read;
use std::io::Write;
use std::io::Seek;

// byteorder
use byteorder::ByteOrder;
use byteorder::LittleEndian;

// Macros

use serde::Serialize;
use serde::Deserialize;

// Types
//--------------------------------------------------------------------------------------------------
	/// The decks table, where all decks are stored
	/// 
	/// # Details
	/// This type serves as an interface to this table, being able to read
	/// and write to it, it is the only type able to do so, as each deck
	/// type may only be converted to and from bytes.
	#[derive(Debug, Serialize, Deserialize)]
	pub struct Table
	{
		decks: Vec<Deck>
	}
	
	#[derive(Debug, Serialize, Deserialize)]
	pub struct Deck
	{
		cards: [u16; 30],
	}
	
	/// Error type for `Table::new`
	#[derive(Debug, derive_more::Display)]
	pub enum TableNewError
	{
		/// Could not seek tothe beginning of the deck table
		#[display(fmt = "Could not seek to the beginning of the deck table")]
		SeekTableBegin( std::io::Error ),
		
		
		
		/// Could not read a deck entry from the deck table
		#[display(fmt = "Unable to fully read a deck entry (The file was too small)")]
		DeckEntry( std::io::Error ),
		
		
		/*
		/// Could not constructs a deck
		#[display(fmt = "Could not construct a deck from the deck table")]
		DeckConstruction( crate::game::deck::deck::FromBytesError ),
		*/
		
		
		/// Could not read the next entry info
		#[display(fmt = "Unable to fully read next entry info (The file was too small)")]
		NextEntryInfo( std::io::Error ),
		
		/*
		/// The deck table was malformed
		#[display(fmt = "The deck table is malformed")]
		MalformedTable( crate::game::deck::property::deck_type::UnknownDeckType ),
		*/
	}
	
	impl std::error::Error for TableNewError {
		fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
			match self {
				Self::SeekTableBegin(err) |
				Self::DeckEntry(err) | 
				Self::NextEntryInfo(err) => Some(err),
			}
		}
	}
	
	/// Error type for `Table::write_to_file`
	#[derive(Debug, derive_more::Display)]
	pub enum TableWriteError
	{
		/// The deck table was too big
		#[display(fmt = "The deck table was too big (is {}, should be 65536 max)", _0)]
		TooManyDeck( usize ),
		
		/*
		/// Unable to convert a deck to bytes
		#[display(fmt = "Unable to convert deck with id {} to bytes", id)]
		UnableToConvertDeckToBytes {
			id: u16,
			err: crate::game::deck::deck::ToBytesError,
		},
		
		/// Unable to write deck entry
		#[display(fmt = "Unable to write deck entry with id {}", id)]
		UnableToWriteDeckEntry {
			id: u16,
			err: std::io::Error,
		},
		*/
	}
//--------------------------------------------------------------------------------------------------

// Impl
//--------------------------------------------------------------------------------------------------
	impl Table
	{
		// Constants
		//--------------------------------------------------------------------------------------------------
			/// The start address of the decks table
			const DECK_TABLE_START_ADDRESS : Data = Data::from_u64(0x21a6808);
		//--------------------------------------------------------------------------------------------------
		
		// Constructors
		//--------------------------------------------------------------------------------------------------
			/// Reads the deck table from a dcb bin file
			pub fn new<F>(game_file: &mut GameFile<F>) -> Result<Table, TableNewError>
			where
				F: Read + Write + Seek
			{
				// The deck array
				let mut decks = vec![];
				
				
				// Seek to the beginning of the deck table
				game_file.seek( std::io::SeekFrom::Start( u64::from( Table::DECK_TABLE_START_ADDRESS) ) ).map_err(TableNewError::SeekTableBegin)?;
				
				// Then loop until we're at the end of the table
				//'table_loop: loop
				for _ in 0..100
				{
					// Read the deck
					let mut buf = [0u8; 110];
					game_file.read_exact(&mut buf).map_err(TableNewError::DeckEntry).unwrap();
					
					// And construct the deck
					let deck = Deck {
						cards: {
							let mut cards_buf = [0u16; 30];
							
							for card_id in 0..30
							{
								cards_buf[card_id] = LittleEndian::read_u16( &buf[0x0 + card_id*2 .. 0x2 + card_id*2] );
							}
							
							cards_buf
						}
					};
					
					// And add it
					decks.push(deck);
				}
				
				// And return the table
				Ok( Table {
					decks,
				})
			}
		//--------------------------------------------------------------------------------------------------
		
		// Write
		//--------------------------------------------------------------------------------------------------
			/// Writes this table to a dcb bin file
			pub fn write_to_file<F>(&self, _game_file: &mut GameFile<F>) -> Result<(), TableWriteError>
			where
				F: Read + Write + Seek
			{
				/*
				// If the table length is bigger than 0xFFFF, return err
				if self.decks.len() > 0xFFFF { return Err( TableWriteError::TooManyDeck( self.decks.len() ) ); }
				
				// Go through all deck and write them
				// Note: We write them in the order they appear in the array,
				//       because this is the same way we read them.
				for (id, deck) in self.decks.iter().enumerate()
				{
					// Convert `id` to a u16
					let id = id as u16;
					
					// Get the bytes
					let mut bytes = [0u8; Deck::BUF_BYTE_SIZE];
					deck.to_bytes(&mut bytes).map_err(|err| TableWriteError::UnableToConvertDeckToBytes{id, err})?;
					
					// Seek to the right address in the table
					Self::seek_deck_table(game_file, id as u16)?;
					
					// And write the deck buffer
					game_file.write_all(&bytes).map_err(|err| TableWriteError::UnableToWriteDeckEntry{id, err})?;
				}
				*/
				
				Ok(())
			}
		//--------------------------------------------------------------------------------------------------
	}
//--------------------------------------------------------------------------------------------------
