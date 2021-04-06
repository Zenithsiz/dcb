//! Compiler

#![feature(box_syntax, backtrace, panic_info_message, array_chunks, format_args_capture, bindings_after_at)]

// Modules
mod cli;

// Imports
use anyhow::Context;

fn main() -> Result<(), anyhow::Error> {
	// Initialize the logger
	simplelog::TermLogger::init(log::LevelFilter::Info, simplelog::Config::default(), simplelog::TerminalMode::Stderr)
		.expect("Unable to initialize logger");

	// Get all data from cli
	let cli = cli::CliData::new();

	// Open the input and output file
	let _input_file = std::fs::File::open(&cli.input_path).context("Unable to open input file")?;
	let _output_file = std::fs::File::open(&cli.output_file_path).context("Unable to open output file")?;

	// Read the executable
	/*
	let exe = Exe::new();

	exe.serialize(&mut input_file).context("Unable to parse game executable")?;
	*/

	Ok(())
}
