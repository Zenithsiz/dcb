//! Compiler

#![feature(box_syntax, backtrace, panic_info_message, array_chunks, format_args_capture, bindings_after_at)]

// Modules
mod cli;

// Imports
use anyhow::Context;
use dcb_exe::inst::parse::{InstParser, ParsedArg, ParsedInst, ParsedLabel};
use dcb_util::{BTreeMapParIter, SignedHex};
use itertools::{Itertools, Position};
use std::{
	collections::BTreeMap,
	io::{BufReader, Write},
};

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
	let lines = InstParser::new(input_file)
		.enumerate()
		.map(|(n, res)| res.map(|line| (n, line)).map_err(|err| (n, err)));
	let res = itertools::process_results(lines, |lines| {
		let mut labels = BTreeMap::new();
		let mut insts = BTreeMap::new();

		for (n, line) in lines {
			if let Some(label) = line.label {
				assert!(labels.insert(n, label).is_none());
			}
			if let Some(inst) = line.inst {
				assert!(insts.insert(n, inst).is_none());
			}
		}

		(labels, insts)
	});
	let (labels, insts): (BTreeMap<usize, ParsedLabel>, BTreeMap<usize, ParsedInst>) = match res {
		Ok(v) => v,
		Err((n, err)) => return Err(err).context(format!("Unable to process line {}", n + 1)),
	};

	// For each instruction, output it
	for (_, line) in BTreeMapParIter::new(&labels, &insts) {
		let (label, inst) = line.into_opt_pair();

		if let Some(label) = &label {
			let padding = if inst.is_some() { " " } else { "" };
			write!(output_file, "{}:{}", label.name, padding).context("Unable to write to output file")?;
		}

		if let Some(inst) = &inst {
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
