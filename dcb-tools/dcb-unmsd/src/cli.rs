//! Cli manager

// Imports
use clap::{App as ClapApp, Arg as ClapArg};
use std::path::PathBuf;

/// Data from the command line
#[derive(PartialEq, Clone, Debug)]
pub struct CliData {
	/// Input files
	pub input_file: PathBuf,
}

impl CliData {
	/// Constructs all of the cli data given and returns it
	pub fn new() -> Self {
		const INPUT_FILE_STR: &str = "input-files";

		// Get all matches from cli
		let matches = ClapApp::new("MSD Extractor")
			.version("0.0")
			.author("Filipe Rodrigues <filipejacintorodrigues1@gmail.com>")
			.about("Extracts the text in a `msd` file")
			.arg(
				ClapArg::with_name(INPUT_FILE_STR)
					.help("The input file to use")
					.required(true)
					.multiple(true),
			)
			.get_matches();

		// Get the input filename
		// Note: required
		let input_file = matches
			.value_of(INPUT_FILE_STR)
			.map(PathBuf::from)
			.expect("Unable to get required argument `input-file`");

		// Return the data
		Self { input_file }
	}
}
