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
mod panic;

// Exports
use cli::CliData;

// Dcb
use dcb::{game::card::Table as CardTable, GameFile};

// Errors
use err_ext::ResultExt;
use err_panic::ErrorExtPanic;

fn main() {
	// Initialize the logger and set the panic handler
	init_logger();
	std::panic::set_hook(box panic::log_handler);

	// Get all data from cli
	let CliData { game_file_path, input_dir } = CliData::new();

	// Load the card table
	let cards_table_file = std::fs::File::open(input_dir.join("cards.yaml")).panic_err_msg("Unable to open `cards.yaml`");
	let cards_table: CardTable = serde_yaml::from_reader(cards_table_file).panic_err_msg("Unable to parse `cards.yaml`");

	// Open the game file
	let game_file = std::fs::OpenOptions::new()
		.write(true)
		.truncate(false)
		.open(game_file_path)
		.panic_err_msg("Unable to open game file");
	let mut game_file = GameFile::from_reader(game_file).panic_err_msg("Unable to initialize game file");

	// And write the cards table
	cards_table.serialize(&mut game_file).panic_err_msg("Unable to serialize cards table");
}

/// Initializes the global logger
fn init_logger() {
	use log::LevelFilter::{Info, Trace};
	use simplelog::{CombinedLogger, Config, SharedLogger, TermLogger, TerminalMode, WriteLogger};
	use std::convert::identity;
	/// The type of logger required to pass to `CombinedLogger::init`
	type BoxedLogger = Box<dyn SharedLogger>;

	// All loggers to try and initialize
	let loggers: Vec<Option<BoxedLogger>> = vec![
		TermLogger::new(Info, Config::default(), TerminalMode::Mixed).map(|logger| BoxedLogger::from(logger)),
		std::fs::File::create("latest.log")
			.ok()
			.map(|file| WriteLogger::new(Trace, Config::default(), file))
			.map(|logger| BoxedLogger::from(logger)),
	];

	// Filter all logger that actually work and initialize them
	CombinedLogger::init(loggers.into_iter().filter_map(identity).collect())
		.ignore_with_err(|_| log::warn!("Logger was already initialized at the start of the program"));
}
