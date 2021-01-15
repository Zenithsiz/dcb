//! Command line arguments

// Imports
use clap::{App as ClapApp, Arg as ClapArg};
use std::path::{Path, PathBuf};

/// Command line data
#[derive(PartialEq, Clone, Debug)]
pub struct CliData {
	/// The game file
	pub game_file_path: PathBuf,

	/// If instruction positions should be printed
	pub print_inst_pos: bool,

	/// If the header should be printed
	pub print_header: bool,
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
			.arg(ClapArg::with_name("print-inst-pos").help("If instructions' positions should be printed"))
			.arg(ClapArg::with_name("print-header").help("If the header of the executable should be printed"))
			.get_matches();

		// Get the input filename
		// Note: required
		let game_file_path = matches
			.value_of("GAME_FILE")
			.map(Path::new)
			.map(Path::to_path_buf)
			.expect("Unable to get required argument `GAME_FILE`");

		let print_inst_pos = matches.is_present("print-inst-pos");
		let print_header = matches.is_present("print-header");

		// Return the cli data
		Self {
			game_file_path,
			print_inst_pos,
			print_header,
		}
	}
}
