//! Compiler

#![feature(box_syntax, backtrace, panic_info_message, array_chunks, format_args_capture, bindings_after_at)]

// Modules
mod cli;

// Imports
use anyhow::Context;
use dcb_exe::inst::parse::InstParser;
use std::io::Write;

fn main() -> Result<(), anyhow::Error> {
	// Initialize the logger
	simplelog::TermLogger::init(log::LevelFilter::Info, simplelog::Config::default(), simplelog::TerminalMode::Stderr)
		.expect("Unable to initialize logger");

	// Get all data from cli
	let cli = cli::CliData::new();

	// Open the input and output file
	let input_file = std::fs::File::open(&cli.input_path).context("Unable to open input file")?;
	let mut output_file = std::fs::File::create(&cli.output_file_path).context("Unable to open output file")?;

	// Read the input
	let parser = InstParser::new(input_file);

	// For each instruction, output it
	for inst in parser {
		writeln!(output_file, "{:?}", inst).context("Unable to write to output file")?;
	}
	/*
	let exe = Exe::new();

	exe.serialize(&mut input_file).context("Unable to parse game executable")?;
	*/

	Ok(())
}
