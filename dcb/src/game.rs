//! Game Data
//!
//! The game module is where all of the game data is defined, this game data
//! can read from the [`GameFile`](crate::io::GameFile) in the `io` module.
//!
//! Some notable types within this module are [`CardTable`](crate::game::card::Table), the table which
//! stores all cards and [`DeckTable`](crate::game::deck::Table), the table which stores all cards available.
//!
//! # Strings
//! A lot of types in this module have strings that they must read and write from the game.
//! All these strings must only contain ascii characters, thus on read and on write, if any
//! strings contain non-ascii characters, an error will occur

// Modules
pub mod card;
pub mod deck;
//pub mod exe;
pub mod validation;

// Exports
pub use card::{Digimon, Digivolve, Item, Table as CardTable};
pub use deck::{Deck, Table as DeckTable};
//pub use exe::{Exe, Header as ExeHeader, Pos as ExePos};
pub use validation::{Validatable, Validation};
