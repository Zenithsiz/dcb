//! Decks

// byteorder
use byteorder::{ByteOrder, LittleEndian};

// Crate
use crate::game::{util, Bytes};

/// A deck
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
pub struct Deck {
	/// Name of this deck
	pub name: ascii::AsciiString,

	/// Digimon who plays this deck
	pub owner: ascii::AsciiString,

	/// All of the card ids that make up this deck
	pub cards: [u16; 30],

	/// Unknown data
	unknown: [u8; 0xc],
}

/// Error type for [`Bytes::from_bytes`]
#[derive(PartialEq, Eq, Clone, Copy, Debug, thiserror::Error)]
pub enum FromBytesError {
	/// Unable to read the deck name
	#[error("Unable to read the deck name")]
	Name(#[source] util::ReadMaybeNullAsciiStringError),

	/// Unable to read the deck owner
	#[error("Unable to read the deck owner")]
	Owner(#[source] util::ReadMaybeNullAsciiStringError),
}

/// Error type for [`Bytes::to_bytes`]
#[derive(PartialEq, Eq, Clone, Copy, Debug, thiserror::Error)]
pub enum ToBytesError {
	/// Unable to write the deck name
	#[error("Unable to write the deck name")]
	Name(#[source] util::WriteMaybeNullAsciiStringError),

	/// Unable to write the deck owner
	#[error("Unable to write the deck owner")]
	Owner(#[source] util::WriteMaybeNullAsciiStringError),
}

impl Bytes for Deck {
	type ByteArray = [u8; 0x6e];
	type FromError = FromBytesError;
	type ToError = ToBytesError;

	fn from_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::FromError> {
		// Split the bytes
		let bytes = util::array_split!(bytes,
			deck   : [0x3c],
			name   : [0x13],
			owner  : [0x13],
			unknown: [0xc],
		);

		Ok(Self {
			name: util::read_maybe_null_ascii_string(bytes.name)
				.map_err(FromBytesError::Name)?
				.to_ascii_string(),

			owner: util::read_maybe_null_ascii_string(bytes.owner)
				.map_err(FromBytesError::Owner)?
				.to_ascii_string(),

			cards: {
				let mut cards_buf = [0; 0x1e];

				for (card_id, card) in cards_buf.iter_mut().enumerate() {
					*card = LittleEndian::read_u16(&bytes.deck[0x0 + card_id * 2..0x2 + card_id * 2]);
				}

				cards_buf
			},

			unknown: *bytes.unknown,
		})
	}

	fn to_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::ToError> {
		// Split the bytes
		let bytes = util::array_split_mut!(bytes,
			deck   : [0x3c],
			name   : [0x13],
			owner  : [0x13],
			unknown: [0xc],
		);

		// Name / Owner
		util::write_maybe_null_ascii_string(&self.name, bytes.name).map_err(ToBytesError::Name)?;
		util::write_maybe_null_ascii_string(&self.owner, bytes.owner).map_err(ToBytesError::Owner)?;

		// Deck
		for (card_id, card) in self.cards.iter().enumerate() {
			LittleEndian::write_u16(&mut bytes.deck[0x0 + card_id * 2..0x2 + card_id * 2], *card);
		}

		// Unknown
		*bytes.unknown = self.unknown;

		// And return Ok
		Ok(())
	}
}
