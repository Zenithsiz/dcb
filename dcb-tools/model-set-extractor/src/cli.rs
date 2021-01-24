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
		let matches = ClapApp::new("Model set Extractor")
			.version("0.0")
			.author("Filipe [...] <[...]@gmail.com>")
			.about("Extracts a model set")
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

		// Try to get the output, else try the filename without extension if it has one, else use `.`
		let output_dir = match matches.value_of("OUTPUT") {
			Some(output) => PathBuf::from(output),
			None => match input_file.extension() {
				Some(_) => input_file.with_extension(""),
				None => Path::new(".").to_path_buf(),
			},
		};

		// Return the data
		Self { input_file, output_dir }
	}
}
