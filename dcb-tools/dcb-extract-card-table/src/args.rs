//! Command line arguments

// Imports
use clap::{App as ClapApp, Arg as ClapArg};
use std::path::PathBuf;

/// Arguments
#[derive(PartialEq, Clone, Debug)]
pub struct Args {
	/// The game file
	pub game_path: PathBuf,
}

impl Args {
	/// Returns all arguments
	pub fn get() -> Self {
		const GAME_FILE_STR: &str = "game-file";

		// Get all matches from cli
		let matches = ClapApp::new("Dcb card table extractor")
			.version("0.0")
			.author("Filipe [...] <[...]@gmail.com>")
			.about("Extracts the card table from the game and prints it to stdout")
			.arg(
				ClapArg::with_name(GAME_FILE_STR)
					.help("The game file")
					.required(true)
					.index(1),
			)
			.get_matches();

		let game_path = matches
			.value_of(GAME_FILE_STR)
			.map(PathBuf::from)
			.expect("Unable to get required argument");

		// Return the cli data
		Self { game_path }
	}
}
