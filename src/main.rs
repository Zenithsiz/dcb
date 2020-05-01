//! Data extractor
//! 
//! # Details
//! Extracts data from the game file to several other files, that can be
//! edited and then used by `Patcher` to modify the game file.
//! 
//! # Syntax
//! The executable may be called as `./dcb-extractor <game file> {-o <output directory>}`
//! 
//! Use the command `./dcb-extractor --help` for more information.
//! 
//! # Data extracted
//! Currently only the following is extracted:
//! - Card table
//! - Deck table (partial)

// Features
#![feature(
	box_syntax,
	backtrace,
	panic_info_message,
)]

// Lints
#![warn(
	clippy::restriction,
	clippy::pedantic,
	clippy::nursery,
)]
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
use dcb::{
	GameFile,
	game::card::Table as CardTable,
};

// Errors
use err_ext::ResultExt;
use err_panic::ErrorExtPanic;
use err_backtrace::ErrBacktraceExt;

fn main() {
	// Initialize the logger and set the panic handler
	init_logger();
	std::panic::set_hook(box panic::log_handler);
	
	// Get all data from cli
	let CliData { input_filename, output_dir } = CliData::new();
	
	// Open the game file
	let input_file = std::fs::File::open(&input_filename)
		.panic_err_msg("Unable to open input file");
	let mut game_file = GameFile::from_reader(input_file)
		.panic_err_msg("Unable to parse input file as dcb");
	
	// Get the cards table
	let cards_table = CardTable::deserialize(&mut game_file)
		.panic_err_msg("Unable to deserialize cards table from game file");
	let cards_table_yaml = serde_yaml::to_string(&cards_table)
		.panic_err_msg("Unable to serialize cards table to yaml");
	
	log::info!("Extracted {} cards", cards_table.card_count());
	
	// And output everything to the files
	let cards_table_output_filename = output_dir.join("cards.yaml");
	std::fs::write(&cards_table_output_filename, cards_table_yaml)
		.map_err(|err| log::warn!("Unable to write output file {}:\n{}", cards_table_output_filename.display(), err.err_backtrace() ))
		.ignore();
}

/// Initializes the global logger
fn init_logger() {
	use simplelog::{CombinedLogger, SharedLogger, TermLogger, WriteLogger, Config, TerminalMode};
	use log::LevelFilter::{Info, Trace};
	use std::convert::identity;
	/// The type of logger required to pass to `CombinedLogger::init`
	type BoxedLogger = Box<dyn SharedLogger>;
	
	// All loggers to try and initialize
	let loggers: Vec< Option<BoxedLogger> > = vec![
		TermLogger ::new(Info, Config::default(), TerminalMode::Mixed)
			.map(|logger| BoxedLogger::from(logger)),
		std::fs::File::create("latest.log").ok()
			.map(|file| WriteLogger::new(Trace, Config::default(), file))
			.map(|logger| BoxedLogger::from(logger))
	];
	
	// Filter all logger that actually work and initialize them
	CombinedLogger::init(
		loggers.into_iter().filter_map(identity).collect()
	).ignore_with_err(|_| log::warn!("Logger was already initialized at the start of the program"));
}
