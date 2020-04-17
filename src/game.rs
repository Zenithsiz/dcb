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

// Lints
//#![allow(clippy::missing_docs_in_private_items)] // A lot of our private items are simple digimon types, so they don't need documentation

// Modules
#[macro_use] pub mod util;

pub mod bytes;
pub mod card;
pub mod deck;



// Exports
pub use bytes::Bytes;
pub use card::Digimon;
