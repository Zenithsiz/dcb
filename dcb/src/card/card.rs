//! A card

// Modules
mod error;

// Exports
pub use error::{DeserializeError, SerializeError};

// Imports
use super::property::CardType;
use crate::{Digimon, Digivolve, Item};
use dcb_bytes::{BytesReadExt, BytesWriteExt};
use std::io;
use zutil::AsciiStrArr;

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
			CardType::Digimon => reader.read_deserialize().map(Self::Digimon)?,
			CardType::Item => reader.read_deserialize().map(Self::Item)?,
			CardType::Digivolve => reader.read_deserialize().map(Self::Digivolve)?,
		};

		Ok(card)
	}

	/// Serializes a card
	pub fn serialize<W: io::Write>(&self, writer: &mut W) -> Result<(), SerializeError> {
		match self {
			Card::Digimon(digimon) => writer.write_serialize(digimon)?,
			Card::Item(item) => writer.write_serialize(item)?,
			Card::Digivolve(digivolve) => writer.write_serialize(digivolve)?,
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

	/// Returns the name of this card mutably
	#[must_use]
	pub fn name_mut(&mut self) -> &mut AsciiStrArr<0x14> {
		match self {
			Self::Digimon(digimon) => &mut digimon.name,
			Self::Item(item) => &mut item.name,
			Self::Digivolve(digivolve) => &mut digivolve.name,
		}
	}

	/// Returns the effect description of this card
	#[must_use]
	pub const fn effect_description(&self) -> &[AsciiStrArr<0x14>; 4] {
		match self {
			Self::Digimon(digimon) => &digimon.effect_description,
			Self::Item(item) => &item.effect_description,
			Self::Digivolve(digivolve) => &digivolve.effect_description,
		}
	}

	/// Returns the effect description of this card mutably
	#[must_use]
	pub fn effect_description_mut(&mut self) -> &mut [AsciiStrArr<0x14>; 4] {
		match self {
			Self::Digimon(digimon) => &mut digimon.effect_description,
			Self::Item(item) => &mut item.effect_description,
			Self::Digivolve(digivolve) => &mut digivolve.effect_description,
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
