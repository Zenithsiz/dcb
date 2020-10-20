//! Data patches
//!
//! # Details
//! Patches data to the game file from several other files.
//!
//! # Syntax
//! The executable may be called as `./patcher <game file> <directory>`
//!
//! Use the command `./patcher --help` for more information.
//!
//! # Data patched
//! Currently only the following is patched:
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
	let CliData { game_file_path, input_dir } = CliData::new();

	// Load the card table
	let cards_table_file = std::fs::File::open(input_dir.join("cards.yaml")).panic_err_msg("Unable to open `cards.yaml`");
	let cards_table: CardTable = serde_yaml::from_reader(cards_table_file).panic_err_msg("Unable to parse `cards.yaml`");

	// Load the decks table
	let decks_table_file = std::fs::File::open(input_dir.join("decks.yaml")).panic_err_msg("Unable to open `decks.yaml`");
	let decks_table: DeckTable = serde_yaml::from_reader(decks_table_file).panic_err_msg("Unable to parse `decks.yaml`");

	// Open the game file
	let game_file = std::fs::OpenOptions::new()
		.write(true)
		.truncate(false)
		.open(game_file_path)
		.panic_err_msg("Unable to open game file");
	let mut game_file = GameFile::from_reader(game_file).panic_err_msg("Unable to initialize game file");

	// And write everything
	cards_table.serialize(&mut game_file).panic_err_msg("Unable to serialize cards table");
	decks_table.serialize(&mut game_file).panic_err_msg("Unable to serialize decks table");
}
