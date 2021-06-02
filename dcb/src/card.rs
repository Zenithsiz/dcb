//! Cards
//!
//! This module contains all cards and card properties that are stored within the game,
//! as well as the card table itself, of all of the cards in the game.

// Modules
#[allow(clippy::module_inception)] // We need to put the `Card` type somewhere
pub mod card;
pub mod digimon;
pub mod digivolve;
pub mod header;
pub mod item;
pub mod property;
pub mod table;

// Exports
pub use card::Card;
pub use digimon::Digimon;
pub use digivolve::Digivolve;
pub use header::CardHeader;
pub use item::Item;
pub use table::Table;
