//! Decks

// Imports
use crate::{
	game::Bytes,
	util::{
		array_split, array_split_mut,
		null_ascii_string::{self, NullAsciiString},
	},
};
use byteorder::{ByteOrder, LittleEndian};

/// Card id type
pub type CardId = u16;

/// A deck
#[derive(PartialEq, Eq, Clone, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Deck {
	/// Name of this deck
	pub name: ascii::AsciiString,

	/// Digimon who plays this deck
	pub owner: ascii::AsciiString,

	/// All of the card ids that make up this deck
	pub cards: [CardId; 30],

	/// Unknown data at `0x62`
	unknown_62: [u8; 0xc],
}

/// Error type for [`Bytes::from_bytes`]
#[derive(PartialEq, Eq, Clone, Copy, Debug, thiserror::Error)]
pub enum FromBytesError {
	/// Unable to read the deck name
	#[error("Unable to read the deck name")]
	Name(#[source] null_ascii_string::ReadError),

	/// Unable to read the deck owner
	#[error("Unable to read the deck owner")]
	Owner(#[source] null_ascii_string::ReadError),
}

/// Error type for [`Bytes::to_bytes`]
#[derive(PartialEq, Eq, Clone, Copy, Debug, thiserror::Error)]
pub enum ToBytesError {
	/// Unable to write the deck name
	#[error("Unable to write the deck name")]
	Name(#[source] null_ascii_string::WriteError),

	/// Unable to write the deck owner
	#[error("Unable to write the deck owner")]
	Owner(#[source] null_ascii_string::WriteError),
}

impl Bytes for Deck {
	type ByteArray = [u8; 0x6e];
	type FromError = FromBytesError;
	type ToError = ToBytesError;

	fn from_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::FromError> {
		// Split the bytes
		let bytes = array_split!(bytes,
			deck      : [0x3c],
			name      : [0x13],
			owner     : [0x13],
			unknown_62: [0xc],
		);

		let mut cards = [0; 30];
		for (card_id, card) in cards.iter_mut().enumerate() {
			/// Size of [`CardId`]
			const CARD_ID_SIZE: usize = std::mem::size_of::<CardId>();
			let offset = card_id * CARD_ID_SIZE;
			*card = LittleEndian::read_u16(&bytes.deck[offset..offset + CARD_ID_SIZE]);
		}

		Ok(Self {
			name: bytes.name.read_string().map_err(FromBytesError::Name)?.to_ascii_string(),
			owner: bytes.owner.read_string().map_err(FromBytesError::Owner)?.to_ascii_string(),
			cards,
			unknown_62: *bytes.unknown_62,
		})
	}

	fn to_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::ToError> {
		// Split the bytes
		let bytes = array_split_mut!(bytes,
			deck      : [0x3c],
			name      : [0x13],
			owner     : [0x13],
			unknown_62: [0xc],
		);

		// Name / Owner
		bytes.name.write_string(&self.name).map_err(ToBytesError::Name)?;
		bytes.owner.write_string(&self.owner).map_err(ToBytesError::Owner)?;

		// Deck
		for (card_id, card) in self.cards.iter().enumerate() {
			/// Size of [`CardId`]
			const CARD_ID_SIZE: usize = std::mem::size_of::<CardId>();
			let offset = card_id * CARD_ID_SIZE;
			LittleEndian::write_u16(&mut bytes.deck[offset..offset + CARD_ID_SIZE], *card);
		}

		// Unknown
		*bytes.unknown_62 = self.unknown_62;

		// And return Ok
		Ok(())
	}
}
