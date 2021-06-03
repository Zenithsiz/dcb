//! Cli manager

// Imports
use clap::{App as ClapApp, Arg as ClapArg};
use std::path::PathBuf;

/// Data from the command line
#[derive(PartialEq, Clone, Debug)]
pub struct CliData {
	/// Input files
	pub input_files: Vec<PathBuf>,

	/// The output file
	pub output_file: Option<PathBuf>,
}

impl CliData {
	/// Constructs all of the cli data given and returns it
	pub fn new() -> Self {
		const INPUT_FILES_STR: &str = "input-files";
		const OUTPUT_FILE_STR: &str = "output-file";

		// Get all matches from cli
		let matches = ClapApp::new("Tmd Extractor")
			.version("0.0")
			.author("Filipe Rodrigues <filipejacintorodrigues1@gmail.com>")
			.arg(
				ClapArg::with_name(INPUT_FILES_STR)
					.help("The input files to use")
					.required(true)
					.multiple(true),
			)
			.arg(
				ClapArg::with_name(OUTPUT_FILE_STR)
					.help("The output file")
					.short("o")
					.long("output-file")
					.takes_value(true),
			)
			.get_matches();

		// Get the input filename
		// Note: required
		let input_files: Vec<_> = matches
			.values_of(INPUT_FILES_STR)
			.expect("Unable to get required argument `input-file`")
			.map(PathBuf::from)
			.collect();


		let output_file = matches.value_of(OUTPUT_FILE_STR).map(PathBuf::from);

		// Return the data
		Self {
			input_files,
			output_file,
		}
	}
}
