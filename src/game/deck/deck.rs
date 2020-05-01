//! Decks

/// A deck
#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
pub struct Deck {
	pub cards: [u16; 30],
}
