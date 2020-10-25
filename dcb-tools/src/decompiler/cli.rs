//! Cli manager

// Imports
use clap::{App as ClapApp, Arg as ClapArg};
use std::path::{Path, PathBuf};

/// Data from the command line
#[derive(PartialEq, Clone, Debug)]
pub struct CliData {
	/// The game file
	pub game_file_path: PathBuf,
}

impl CliData {
	/// Constructs all of the cli data given and returns it
	pub fn new() -> Self {
		// Get all matches from cli
		let matches = ClapApp::new("Dcb Decompiler")
			.version("0.0")
			.author("Filipe [...] <[...]@gmail.com>")
			.about("Decompiles all code from the Digimon Digital Card Battle `.bin` game file")
			.arg(
				ClapArg::with_name("GAME_FILE")
					.help("Sets the input game file to use")
					.required(true)
					.index(1),
			)
			.get_matches();

		// Get the input filename
		// Note: required
		let game_file_path = matches
			.value_of("GAME_FILE")
			.map(Path::new)
			.map(Path::to_path_buf)
			.expect("Unable to get required argument `GAME_FILE`");

		// Return the cli data
		Self { game_file_path }
	}
}
