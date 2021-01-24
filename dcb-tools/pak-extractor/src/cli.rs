//! Cli manager

// Imports
use clap::{App as ClapApp, Arg as ClapArg};
use std::path::{Path, PathBuf};

/// Data from the command line
#[derive(PartialEq, Clone, Debug)]
pub struct CliData {
	/// Input file
	pub input_file: PathBuf,

	/// The output directory
	pub output_dir: PathBuf,
}

impl CliData {
	/// Constructs all of the cli data given and returns it
	pub fn new() -> Self {
		// Get all matches from cli
		let matches = ClapApp::new("Pak Extractor")
			.version("0.0")
			.author("Filipe [...] <[...]@gmail.com>")
			.about("Extracts the filesystem in a `.pak` filesystem")
			.arg(
				ClapArg::with_name("INPUT_FILE")
					.help("Sets the input file to use")
					.required(true)
					.index(1),
			)
			.arg(
				ClapArg::with_name("OUTPUT")
					.help("Sets the output directory to use")
					.short("o")
					.long("output")
					.takes_value(true)
					.required(false),
			)
			.get_matches();

		// Get the input filename
		// Note: required
		let input_file = matches
			.value_of("INPUT_FILE")
			.map(Path::new)
			.map(Path::to_path_buf)
			.expect("Unable to get required argument `INPUT_FILE`");

		// Try to get the output, else just use `.`
		let output_dir = match matches.value_of("OUTPUT") {
			Some(output) => PathBuf::from(output),
			None => Path::new(".").to_path_buf(),
		};

		// Return the data
		Self { input_file, output_dir }
	}
}
