//! `dcb` is a library for interacting with the game file of `Digimon Digital Card Battle`,
//! a PSX game.
//!
//! # Modules
//! `dcb` splits itself into 2 main modules, [`io`], which interacts with the game file
//! as well as general input / output operations and [`game`], where most of  the game's
//! data types are defined.
//!
//! # Example
//! 
//! The following is an example of how to use the `dcb` library.
//! This example extracts the card table and prints it to screen
//!
//! ```no_run
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! #   use std::fs::File;
//! let mut game_file = dcb::GameFile::from_reader( File::open("Digimon Digital Card Battle.bin")? )?;
//! let card_table = dcb::game::card::Table::deserialize( &mut game_file )?;
//! println!("Card table: {:?}", card_table);
//! #   Ok(())
//! # }
//! ```

// Features
#![feature(
	seek_convenience,
	never_type,
	trait_alias,
	unsized_locals,
	bool_to_option,
	decl_macro,
	stmt_expr_attributes,
	unwrap_infallible,
	const_if_match,
	exclusive_range_pattern,
)]

// Lints
#![warn(
	clippy::restriction,
	clippy::pedantic,
	clippy::nursery,
)]
#![allow(
	clippy::missing_inline_in_public_items, // Dubious lint
	clippy::implicit_return,                // We prefer tail returns where possible
	clippy::shadow_reuse,                   // Very useful for arguments `arg: impl Into<U>; let arg = arg.into()`
	clippy::if_not_else,                    // Sometimes it's easier to read with a negation
	clippy::result_expect_used,
	clippy::option_expect_used,             // We use `.expect` when there is no safe alternative and the program is corrupt
	clippy::unreadable_literal,             // More important to be able to copy the number with no formatting than it being readable
	clippy::multiple_inherent_impl,         // We prefer to separate certain methods by type and insert error types in between methods
	clippy::identity_op,                    // Makes sense sometimes for symmetry
	clippy::items_after_statements,         // Sometimes we only introduce items when we first use them.
	clippy::unseparated_literal_suffix,     // We only separate them when they are long
	clippy::match_same_arms,                // Sometimes we separate them for clarify and order
	clippy::missing_errors_doc,             // We provide documentation on errors on the error type itself
	clippy::todo,                           // Code that is incomplete should be tagged as such.
	clippy::unreachable,                    // Some code should be unreachable and panic when reached.
	clippy::integer_arithmetic,             // Come on now, we need to use numbers to program
	clippy::shadow_same,                    // Useful when taking arguments such as `value: impl AsRef<T>` / `let value = value.as_ref();`
	
	// TODO: Deal with casts eventually
	clippy::cast_possible_wrap,
	clippy::cast_sign_loss,
	clippy::cast_possible_truncation,
	
	// TODO: Remove these once all modules are ported
	clippy::missing_docs_in_private_items,
	clippy::as_conversions,
	clippy::indexing_slicing,
	
)]

// Modules
pub mod io;
pub mod game;

// Exports
pub use io::GameFile;
