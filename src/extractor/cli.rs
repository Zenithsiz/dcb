//! Cli manager for the extractor

// Filesystem
use std::path::{Path, PathBuf};

// Clap
use clap::{App as ClapApp, Arg as ClapArg};

// Errors
use err_panic::ErrorExtPanic;

/// Data from the command line
#[derive(PartialEq, Clone, Debug)]
pub struct CliData {
	/// The game file
	pub game_file_path: PathBuf,

	/// The ouput directory
	pub output_dir: PathBuf,
}

impl CliData {
	/// Constructs all of the cli data given and returns it
	pub fn new() -> Self {
		// Get all matches from cli
		let matches = ClapApp::new("Dcb Extractor")
			.version("0.0")
			.author("Filipe [...] <[...]@gmail.com>")
			.about("Extracts all data from a Digimon Digital Card Battle `.bin` game file")
			.arg(
				ClapArg::with_name("GAME_FILE")
					.help("Sets the input game file to use")
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
		let game_file_path = matches
			.value_of("GAME_FILE")
			.map(Path::new)
			.map(Path::to_path_buf)
			.panic_msg("Unable to get required argument `GAME_FILE`");

		// Try to get the output
		let output_dir = match matches.value_of("OUTPUT") {
			Some(output) => PathBuf::from(output),
			None => game_file_path.parent().unwrap_or_else(|| Path::new(".")).to_path_buf(),
		};

		// Return the cli data
		Self { game_file_path, output_dir }
	}
}
