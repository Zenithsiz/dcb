//! Decks

// Imports
use crate::{
	game::{
		deck::{armor_evo, city, music, ArmorEvo, City, Music},
		Bytes,
	},
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

	/// Experience gained by beating this deck
	pub experience: u8,

	/// City of the deck
	pub city: Option<City>,

	/// Armor digivolution
	pub armor_evo: Option<ArmorEvo>,

	/// Battle music
	pub battle_music: Option<Music>,

	/// Polygon music
	pub polygon_music: Option<Music>,

	/// Unknown data at `0x62`
	unknown_64: [u8; 0x4],

	/// Unknown data at `0x6a`
	unknown_6a: u8,
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

	/// Unable to read the deck city
	#[error("Unable to read the deck city")]
	City(#[source] city::FromBytesError),

	/// Unable to read the armor digivolution
	#[error("Unable to read the deck armor digivolution")]
	ArmorEvo(#[source] armor_evo::FromBytesError),

	/// Unable to read the battle music
	#[error("Unable to read the deck battle music")]
	BattleMusic(#[source] music::FromBytesError),

	/// Unable to read the polygon music
	#[error("Unable to read the deck polygon music")]
	PolygonMusic(#[source] music::FromBytesError),
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
			deck         : [0x3c],
			name         : [0x13],
			owner        : [0x15],
			unknown_64   : [0x4],
			battle_music : 1,
			polygon_music: 1,
			city         : 1,
			unknown_6a   : 1,
			experience   : 1,
			armor_evo    : 1,
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
			city: Option::<City>::from_bytes(bytes.city).map_err(FromBytesError::City)?,
			armor_evo: Option::<ArmorEvo>::from_bytes(bytes.armor_evo).map_err(FromBytesError::ArmorEvo)?,
			battle_music: Option::<Music>::from_bytes(bytes.battle_music).map_err(FromBytesError::BattleMusic)?,
			polygon_music: Option::<Music>::from_bytes(bytes.polygon_music).map_err(FromBytesError::PolygonMusic)?,
			experience: *bytes.experience,
			unknown_64: *bytes.unknown_64,
			unknown_6a: *bytes.unknown_6a,
		})
	}

	fn to_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::ToError> {
		// Split the bytes
		let bytes = array_split_mut!(bytes,
			deck         : [0x3c],
			name         : [0x13],
			owner        : [0x15],
			unknown_64   : [0x4],
			battle_music : 1,
			polygon_music: 1,
			city         : 1,
			unknown_6a   : 1,
			experience   : 1,
			armor_evo    : 1,
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

		// Experience
		*bytes.experience = self.experience;

		// City
		self.city.to_bytes(bytes.city).into_ok();

		// Armor evo
		self.armor_evo.to_bytes(bytes.armor_evo).into_ok();

		// Music
		self.battle_music.to_bytes(bytes.battle_music).into_ok();
		self.polygon_music.to_bytes(bytes.polygon_music).into_ok();

		// Unknown
		*bytes.unknown_64 = self.unknown_64;
		*bytes.unknown_6a = self.unknown_6a;

		// And return Ok
		Ok(())
	}
}
