//! A card

// Modules
pub mod error;

// Exports
pub use error::{DeserializeError, SerializeError};

// Imports
use super::property::CardType;
use crate::{Digimon, Digivolve, Item};
use dcb_bytes::{ByteArray, Bytes};
use dcb_util::AsciiStrArr;
use std::io;

/// A card
#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum Card {
	/// Digimon
	Digimon(Digimon),

	/// Item
	Item(Item),

	/// Digivolve
	Digivolve(Digivolve),
}

impl Card {
	/// Deserializes a card
	pub fn deserialize<R: io::Read>(card_type: CardType, reader: &mut R) -> Result<Self, DeserializeError> {
		let card = match card_type {
			CardType::Digimon => {
				let mut bytes = <Digimon as Bytes>::ByteArray::zeros();
				reader.read_exact(&mut bytes).map_err(DeserializeError::Read)?;
				Digimon::from_bytes(&bytes)
					.map(Self::Digimon)
					.map_err(DeserializeError::ParseDigimon)?
			},
			CardType::Item => {
				let mut bytes = <Item as Bytes>::ByteArray::zeros();
				reader.read_exact(&mut bytes).map_err(DeserializeError::Read)?;
				Item::from_bytes(&bytes)
					.map(Self::Item)
					.map_err(DeserializeError::ParseItem)?
			},
			CardType::Digivolve => {
				let mut bytes = <Digivolve as Bytes>::ByteArray::zeros();
				reader.read_exact(&mut bytes).map_err(DeserializeError::Read)?;
				Digivolve::from_bytes(&bytes)
					.map(Self::Digivolve)
					.map_err(DeserializeError::ParseDigivolve)?
			},
		};

		Ok(card)
	}

	/// Serializes a card
	pub fn serialize<W: io::Write>(&self, writer: &mut W) -> Result<(), SerializeError> {
		match self {
			Card::Digimon(digimon) => {
				let bytes = digimon.bytes().map_err(SerializeError::SerializeDigimon)?;
				writer.write_all(&bytes).map_err(SerializeError::Write)?;
			},
			Card::Item(item) => {
				let bytes = item.bytes().map_err(SerializeError::SerializeItem)?;
				writer.write_all(&bytes).map_err(SerializeError::Write)?;
			},
			Card::Digivolve(digivolve) => {
				let bytes = digivolve.bytes().into_ok();
				writer.write_all(&bytes).map_err(SerializeError::Write)?;
			},
		}

		Ok(())
	}
}

impl Card {
	/// Returns the name of this card
	#[must_use]
	pub const fn name(&self) -> &AsciiStrArr<0x14> {
		match self {
			Self::Digimon(digimon) => &digimon.name,
			Self::Item(item) => &item.name,
			Self::Digivolve(digivolve) => &digivolve.name,
		}
	}

	/// Returns this card's type
	#[must_use]
	pub const fn ty(&self) -> CardType {
		match self {
			Card::Digimon(_) => CardType::Digimon,
			Card::Item(_) => CardType::Item,
			Card::Digivolve(_) => CardType::Digivolve,
		}
	}

	/// Returns `true` if the card is [`Digimon`].
	#[must_use]
	pub const fn is_digimon(&self) -> bool {
		matches!(self, Self::Digimon(..))
	}

	/// Returns `true` if the card is [`Item`].
	#[must_use]
	pub const fn is_item(&self) -> bool {
		matches!(self, Self::Item(..))
	}

	/// Returns `true` if the card is [`Digivolve`].
	#[must_use]
	pub const fn is_digivolve(&self) -> bool {
		matches!(self, Self::Digivolve(..))
	}

	/// Returns this card a digimon
	#[must_use]
	pub const fn as_digimon(&self) -> Option<&Digimon> {
		match self {
			Self::Digimon(v) => Some(v),
			_ => None,
		}
	}

	/// Returns this card an item
	#[must_use]
	pub const fn as_item(&self) -> Option<&Item> {
		match self {
			Self::Item(v) => Some(v),
			_ => None,
		}
	}

	/// Returns this card a digivolve
	#[must_use]
	pub const fn as_digivolve(&self) -> Option<&Digivolve> {
		match self {
			Self::Digivolve(v) => Some(v),
			_ => None,
		}
	}
}
