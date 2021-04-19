//! Command line arguments

// Imports
use clap::{App as ClapApp, Arg as ClapArg};
use std::path::{Path, PathBuf};

/// Command line data
#[derive(PartialEq, Clone, Debug)]
pub struct CliData {
	/// The input file
	pub input_path: PathBuf,

	/// Output file
	pub output_file_path: PathBuf,
}

impl CliData {
	/// Constructs all of the cli data given and returns it
	pub fn new() -> Self {
		const INPUT_FILE_STR: &str = "input-file";
		const OUTPUT_FILE_STR: &str = "output-file";

		// Get all matches from cli
		let matches = ClapApp::new("Dcb Decompiler")
			.version("0.0")
			.author("Filipe [...] <[...]@gmail.com>")
			.about("Compiles code from assembly")
			.arg(
				ClapArg::with_name(INPUT_FILE_STR)
					.help("Sets the input file to use")
					.required(true)
					.index(1),
			)
			.arg(
				ClapArg::with_name(OUTPUT_FILE_STR)
					.long("output")
					.short("o")
					.help("Sets the output file")
					.takes_value(true),
			)
			.get_matches();

		// Get the input filename
		// Note: required
		let input_path = matches
			.value_of(INPUT_FILE_STR)
			.map(Path::new)
			.map(Path::to_path_buf)
			.expect("Unable to get required argument `input-file`");

		// Get the output directory, or just use `a.out`
		let output_file_path = match matches.value_of(OUTPUT_FILE_STR) {
			Some(path) => PathBuf::from(path),
			None => PathBuf::from("a.out"),
		};

		// Return the cli data
		Self {
			input_path,
			output_file_path,
		}
	}
}
