//! Command line arguments

// Imports
use clap::{App as ClapApp, Arg as ClapArg};
use std::path::PathBuf;

/// Arguments
#[derive(PartialEq, Clone, Debug)]
pub struct Args {
	/// The game file
	pub game_path: PathBuf,

	/// The bios path
	pub bios_path: PathBuf,
}

impl Args {
	/// Returns all arguments
	pub fn new() -> Self {
		const GAME_FILE_STR: &str = "game-file";
		const BIOS_STR: &str = "bios";

		// Get all matches from cli
		let matches = ClapApp::new("Dcb Debugger")
			.version("0.0")
			.author("Filipe [...] <[...]@gmail.com>")
			.about("Runs and debugs the game")
			.arg(
				ClapArg::with_name(GAME_FILE_STR)
					.help("Sets the game file")
					.required(true)
					.index(1),
			)
			.arg(
				ClapArg::with_name(BIOS_STR)
					.help("The psx bios")
					.long_help("The psx bios, usually a file called `SCPH1001.BIN`")
					.takes_value(true)
					.required(true)
					.long("bios"),
			)
			.get_matches();

		let game_path = matches
			.value_of(GAME_FILE_STR)
			.map(PathBuf::from)
			.expect("Unable to get required argument");
		let bios_path = matches
			.value_of(BIOS_STR)
			.map(PathBuf::from)
			.expect("Unable to get required argument");

		// Return the cli data
		Self { game_path, bios_path }
	}
}
