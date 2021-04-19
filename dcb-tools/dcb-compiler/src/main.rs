//! Compiler

#![feature(box_syntax, backtrace, panic_info_message, array_chunks, format_args_capture, bindings_after_at)]

// Modules
mod cli;

// Imports
use anyhow::Context;
use dcb_exe::inst::parse::{InstParser, ParsedArg};
use dcb_util::SignedHex;
use itertools::{Itertools, Position};
use std::io::{BufReader, Write};

fn main() -> Result<(), anyhow::Error> {
	// Initialize the logger
	simplelog::TermLogger::init(log::LevelFilter::Info, simplelog::Config::default(), simplelog::TerminalMode::Stderr)
		.expect("Unable to initialize logger");

	// Get all data from cli
	let cli = cli::CliData::new();

	// Open the input and output file
	let input_file = std::fs::File::open(&cli.input_path).context("Unable to open input file")?;
	let input_file = BufReader::new(input_file);
	let mut output_file = std::fs::File::create(&cli.output_file_path).context("Unable to open output file")?;

	// Read the input
	let parser = InstParser::new(input_file);

	// For each instruction, output it
	for (n, line) in parser.enumerate() {
		let line = line.with_context(|| format!("Unable to parse line {}", n + 1))?;

		if line.label.is_none() && line.inst.is_none() {
			continue;
		}

		if let Some(label) = line.label {
			let padding = if line.inst.is_some() { " " } else { "" };
			write!(output_file, "{}:{}", label, padding).context("Unable to write to output file")?;
		}

		if let Some(inst) = line.inst {
			let padding = if inst.args.is_empty() { "" } else { " " };
			write!(output_file, "{}{}", inst.mnemonic, padding).context("Unable to write to output file")?;

			for arg in inst.args.iter().with_position() {
				let is_last = matches!(arg, Position::Last(_) | Position::Only(_));

				match arg.into_inner() {
					ParsedArg::String(s) => write!(output_file, r#""{}""#, s.escape_default()),
					ParsedArg::Literal(num) => write!(output_file, "{:#}", SignedHex(num)),
					ParsedArg::Register(reg) => write!(output_file, "{reg}"),
					ParsedArg::RegisterOffset { register, offset } => write!(output_file, "{:#}({register})", SignedHex(offset)),
					ParsedArg::Label(label) => write!(output_file, "{label}"),
					ParsedArg::LabelOffset { label, offset } => write!(output_file, "{label}+{:#}", SignedHex(offset)),
				}
				.context("Unable to write to output file")?;

				if !is_last {
					write!(output_file, ", ").context("Unable to write to output file")?;
				}
			}
		}

		writeln!(output_file).context("Unable to write to output file")?;
	}

	Ok(())
}
