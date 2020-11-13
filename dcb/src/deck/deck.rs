//! Decks

// Imports
use crate::deck::{armor_evo, city, music, ArmorEvo, City, MaybeArmorEvo, MaybeCity, MaybeMusic, Music};
use byteorder::{ByteOrder, LittleEndian};
use dcb_bytes::Bytes;
use dcb_util::{
	array_split, array_split_mut,
	null_ascii_string::{self, NullAsciiString},
	AsciiStrArr,
};
use ref_cast::RefCast;

/// Card id type
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[derive(derive_more::From, derive_more::Into)]
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct CardId(pub u16);

/// A deck
#[derive(PartialEq, Eq, Clone, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Deck {
	/// Name of this deck
	pub name: AsciiStrArr<0x12>,

	/// Digimon who plays this deck
	pub owner: AsciiStrArr<0x14>,

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

/// Error type for [`Bytes::from_bytes`](dcb_bytes::Bytes::from_bytes)
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

impl Bytes for Deck {
	type ByteArray = [u8; 0x6e];
	type FromError = FromBytesError;
	type ToError = !;

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
		for (card, bytes) in cards.iter_mut().zip(bytes.deck.chunks(2)) {
			*card = LittleEndian::read_u16(bytes);
		}

		Ok(Self {
			name:          bytes.name.read_string().map_err(FromBytesError::Name)?,
			owner:         bytes.owner.read_string().map_err(FromBytesError::Owner)?,
			cards:         cards.map(CardId),
			city:          MaybeCity::from_bytes(bytes.city).map_err(FromBytesError::City)?.into(),
			armor_evo:     MaybeArmorEvo::from_bytes(bytes.armor_evo).map_err(FromBytesError::ArmorEvo)?.into(),
			battle_music:  MaybeMusic::from_bytes(bytes.battle_music).map_err(FromBytesError::BattleMusic)?.into(),
			polygon_music: MaybeMusic::from_bytes(bytes.polygon_music).map_err(FromBytesError::PolygonMusic)?.into(),
			experience:    *bytes.experience,
			unknown_64:    *bytes.unknown_64,
			unknown_6a:    *bytes.unknown_6a,
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
		bytes.name.write_string(&self.name);
		bytes.owner.write_string(&self.owner);

		// Deck
		for (&card, bytes) in self.cards.iter().zip(bytes.deck.chunks_mut(2)) {
			LittleEndian::write_u16(bytes, card.0);
		}

		// Experience
		*bytes.experience = self.experience;

		// City
		MaybeCity::ref_cast(&self.city).to_bytes(bytes.city).into_ok();

		// Armor evo
		MaybeArmorEvo::ref_cast(&self.armor_evo).to_bytes(bytes.armor_evo).into_ok();

		// Music
		MaybeMusic::ref_cast(&self.battle_music).to_bytes(bytes.battle_music).into_ok();
		MaybeMusic::ref_cast(&self.polygon_music).to_bytes(bytes.polygon_music).into_ok();

		// Unknown
		*bytes.unknown_64 = self.unknown_64;
		*bytes.unknown_6a = self.unknown_6a;

		// And return Ok
		Ok(())
	}
}
