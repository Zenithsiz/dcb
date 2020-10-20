//! Data extractor
//!
//! # Details
//! Extracts data from the game file to several other files, that can be
//! edited and then used by `Patcher` to modify the game file.
//!
//! # Syntax
//! The executable may be called as `./extractor <game file> {-o <output directory>}`
//!
//! Use the command `./extractor --help` for more information.
//!
//! # Data extracted
//! Currently only the following is extracted:
//! - Card table

// Features
#![feature(box_syntax, backtrace, panic_info_message)]
// Lints
#![warn(clippy::restriction, clippy::pedantic, clippy::nursery)]
#![allow(
	clippy::implicit_return,         // We prefer implicit returns where possible
	clippy::module_name_repetitions, // This happens often due to separating things into modules finely
	clippy::wildcard_enum_match_arm, // We only use wildcards when we truly only care about some variants
	clippy::result_expect_used,
	clippy::option_expect_used,      // We use expect when there is no alternative.
	clippy::used_underscore_binding, // Useful for macros and such
)]

// Modules
// TODO: `cargo fmt` cannot use this syntax, possibly change it once possible
mod cli;
#[path = "../logger.rs"]
mod logger;
#[path = "../panic.rs"]
mod panic;

// Exports
use cli::CliData;

// Dcb
use dcb::{
	game::{card::Table as CardTable, deck::Table as DeckTable},
	GameFile,
};

// Errors
use err_panic::ErrorExtPanic;

fn main() {
	// Initialize the logger and set the panic handler
	logger::init();
	std::panic::set_hook(box panic::log_handler);

	// Get all data from cli
	let CliData { game_file_path, output_dir } = CliData::new();

	// Open the game file
	let input_file = std::fs::File::open(&game_file_path).panic_err_msg("Unable to open input file");
	let mut game_file = GameFile::from_reader(input_file).panic_err_msg("Unable to parse input file as dcb");

	// Get the cards table
	let cards_table = CardTable::deserialize(&mut game_file).panic_err_msg("Unable to deserialize cards table from game file");
	let cards_table_yaml = serde_yaml::to_string(&cards_table).panic_err_msg("Unable to serialize cards table to yaml");
	log::info!("Extracted {} cards", cards_table.card_count());

	// Get the decks table
	let decks_table = DeckTable::deserialize(&mut game_file).panic_err_msg("Unable to deserialize decks table from game file");
	let decks_table_yaml = serde_yaml::to_string(&decks_table).panic_err_msg("Unable to serialize decks table to yaml");

	// And output everything to the files
	std::fs::write(&output_dir.join("cards.yaml"), cards_table_yaml).panic_err_msg("Unable to write cards table to file");
	std::fs::write(&output_dir.join("decks.yaml"), decks_table_yaml).panic_err_msg("Unable to write decks table to file");
}
