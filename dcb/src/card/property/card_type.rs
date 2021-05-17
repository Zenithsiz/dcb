//! Card type

// Imports
use crate::card::{Digimon, Digivolve, Item};
use dcb_bytes::Bytes;
use std::mem;

/// A card type
#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(strum::IntoStaticStr, strum::Display, strum::EnumIter)]
#[derive(dcb_bytes_derive::Discriminant)]
pub enum CardType {
	/// [`Digimon`]
	Digimon   = 0,

	/// [`Item`]
	Item      = 1,

	/// [`Digivolve`]
	Digivolve = 2,
}


impl CardType {
	/// Returns a string representing this card type
	#[must_use]
	pub fn as_str(self) -> &'static str {
		self.into()
	}

	/// Returns the byte size of the corresponding card
	#[must_use]
	pub const fn byte_size(self) -> usize {
		match self {
			Self::Digimon => mem::size_of::<<Digimon as Bytes>::ByteArray>(),
			Self::Item => mem::size_of::<<Item as Bytes>::ByteArray>(),
			Self::Digivolve => mem::size_of::<<Digivolve as Bytes>::ByteArray>(),
		}
	}
}
