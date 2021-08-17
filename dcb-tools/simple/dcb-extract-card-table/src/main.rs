//! Extracts the card table
//!
//! # Output
//! Outputs results to stdout in json

// Features
#![feature(
	with_options,
	format_args_capture,
	once_cell,
	never_type,
	seek_stream_len,
	try_blocks,
	array_zip,
	unwrap_infallible
)]

// Modules
mod args;

// Imports
use anyhow::Context;
use dcb::CardTable;
use std::{fs, io};

fn main() -> Result<(), anyhow::Error> {
	// Initialize the logger
	simplelog::TermLogger::init(
		log::LevelFilter::Debug,
		simplelog::Config::default(),
		simplelog::TerminalMode::Stderr,
		simplelog::ColorChoice::Auto,
	)
	.context("Unable to initialize logger")?;

	// Get the arguments
	let args = args::Args::get();

	// Open the file
	let file = fs::File::open(&args.game_path)
		.with_context(|| format!("Unable to open game file {}", args.game_path.display()))?;
	let mut file = dcb_cdrom_xa::CdRomCursor::new(file);

	// Open the card table file and parse it
	let mut game_file = dcb_io::GameFile::new(&mut file);
	let mut table_file = CardTable::open(&mut game_file).context("Unable to open table file")?;

	// Then parse it
	let card_table = CardTable::deserialize(&mut table_file).context("Unable to parse card table")?;

	// And print it to stdout.
	serde_json::to_writer_pretty(io::stdout(), &card_table).context("Unable to write")?;

	Ok(())
}
