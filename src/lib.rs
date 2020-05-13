//! `dcb` is a library for interacting with the game file of `Digimon Digital Card Battle`.
//!
//! # Modules
//! `dcb` is split across 2 main modules, [`io`] and [`game`].
//!
//! ## Io
//! The Io module is responsible for interacting with the game file. In the future it may be responsible
//! for also interacting with the game extracted database, once work on that is complete.
//!
//! ## Game
//! The game module is responsible for representing in-game structures such as cards, sprites, text, and
//! others. The trait has various interfaces to be able to deserialize these structures from both the game
//! file, database or even other sources, depending on the structure.
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
	external_doc
)]
// Lints
#![warn(clippy::restriction, clippy::pedantic, clippy::nursery)]
// Necessary items may be inlined using `LTO`, so we don't need to mark them as inline
#![allow(clippy::missing_inline_in_public_items)]
// We prefer tail returns where possible, as they help with code readability in most cases.
#![allow(clippy::implicit_return)]
// Very useful for arguments such as `arg: impl Into<U>`, then used
// with `let arg = arg.into()`. As well as just going from `Option<T>`
// to `T` without needing to change their names.
#![allow(clippy::shadow_reuse, clippy::shadow_same)]
// We use `.expect("...")` when we either know we cannot panic or it
// is the safest alternative, as proceeding would corrupt the program state.
#![allow(clippy::result_expect_used, clippy::option_expect_used)]
// Like-wise with `.expect()`, we use `unreachable!` when we know a branch
// if unreachable, and if it ever does get reached, panicking would be the
// safest option
#![allow(clippy::unreachable)]
// We find it more important to be able to copy paste literals such as `0xabcd1234` than
// being able to read them, which does not provide many benefits
#![allow(clippy::unreadable_literal, clippy::unseparated_literal_suffix)]
// We separate implementations per their functionality usually, such as constructors, getters, setters, and others.
#![allow(clippy::multiple_inherent_impl)]
// Many operations we need to repeat, and to keep symmetry
#![allow(clippy::identity_op)]
// We only introduce items before their first usage, which sometimes is half-way through the code
#![allow(clippy::items_after_statements)]
// Useful for when they either change a lot with new variants / data,
// or for symmetry purposes
#![allow(clippy::match_same_arms)]
// In this library we have very grain-level error types, each function
// will have it's own error type ideally, so any errors are explicit
// by the type, without needing a section for them
#![allow(clippy::missing_errors_doc)]
// Incomplete code should be tagged as `todo`. In future versions of the library,
// this lint may be removed, as incomplete code should not lie on a master branch.
#![allow(clippy::todo)]
// Although we generally try to avoid this, this can happen due to our module organization.
// In the future, this lint should be removed globally and only enabled for modules which
// actually require the use of it.
#![allow(clippy::module_inception)]
// False positives:
// TODO: Remove them in the future once they are no longer triggered.
// We only slice arrays, which are verified at compile time. This
// lint currently triggers for `&[T; N]`, which we pass around a lot.
#![allow(clippy::indexing_slicing)]
// We don't have any `unsafe` impl for types that derive `Deserialize`.
#![allow(clippy::unsafe_derive_deserialize)]
// Banning arithmetic is too strict for this project
#![allow(clippy::integer_arithmetic)]
// TODO: Remove once fixed
#![allow(
	clippy::missing_docs_in_private_items,
	clippy::as_conversions,
	clippy::cast_possible_wrap,
	clippy::cast_sign_loss,
	clippy::cast_possible_truncation
)]

// Modules
pub mod game;
pub mod io;

// Exports
pub use game::{Bytes, CardTable, Deck, DeckTable, Digimon, Digivolve, Item};
pub use io::GameFile;
