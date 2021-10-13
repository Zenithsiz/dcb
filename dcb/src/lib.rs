//! `dcb` is a library for interacting with the game file of `Digimon Digital Card Battle`.

// Features
#![feature(
	never_type,
	stmt_expr_attributes,
	unwrap_infallible,
	format_args_capture,
	iter_is_partitioned,
	try_trait_v2,
	array_zip,
	array_methods,
	iter_zip
)]

// Modules
pub mod card;
pub mod deck;

// Exports
pub use card::{Digimon, Digivolve, Item, Table as CardTable};
pub use deck::{Deck, Table as DeckTable};
