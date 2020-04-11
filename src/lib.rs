//! `dcb` is a library for interacting with the game file of `Digital Card Battle`,
//! a PSX game.
//!
//! # Modules
//! `dcb` split itself into 2 main modules, [io], which interacts with the game file
//! as well as general input / output operations and [game], which is where most of
//! the game's data types are defined.
//!
//! # Example
//! 
//! The following is an example of how to use the `dcb` library.
//! This example extracts the card table and prints it to screen
//!
//! ```no_run
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! #   use std::fs::File;
//!     let mut game_file = dcb::GameFile::from_reader( File::open("resources/Digimon Digital Card Battle.bin")? );
//!     let card_table = dcb::game::card::Table::new( &mut game_file )?;
//!     println!("Card table: {:?}", card_table);
//! #   Ok(())
//! # }
//! ```

// Features
#![feature(seek_convenience)]
#![feature(never_type)]
#![feature(trait_alias)]

// Lints
#![warn(
	clippy::restriction,
	clippy::pedantic,
	clippy::nursery,
)]

// Modules
pub mod io;
pub mod game;

// Exports
pub use io::GameFile;
