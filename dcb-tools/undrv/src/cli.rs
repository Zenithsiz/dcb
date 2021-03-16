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
		let matches = ClapApp::new("Drv Extractor")
			.version("0.1")
			.author("Filipe [...] <[...]@gmail.com>")
			.about("Extracts `.drv` files")
			.arg(ClapArg::with_name("INPUT_FILE").help("The input file to use").required(true).index(1))
			.arg(
				ClapArg::with_name("OUTPUT")
					.help("The directory to output to")
					.long_help(
						"The directory to output to. If not specified, the parent of the input file is used. If it doesn't exist, the current \
						 directory is used",
					)
					.short("d")
					.long("output-dir")
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
