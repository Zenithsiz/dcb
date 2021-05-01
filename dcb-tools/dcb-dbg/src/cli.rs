//! Command line arguments

// Imports
use clap::{App as ClapApp, Arg as ClapArg};
use std::path::PathBuf;

/// Command line data
#[derive(PartialEq, Clone, Debug)]
pub struct CliData {
	/// The input file
	pub input_path: PathBuf,

	/// The header file
	pub header_path: PathBuf,
}

impl CliData {
	/// Constructs all of the cli data given and returns it
	pub fn new() -> Self {
		const INPUT_FILE_STR: &str = "input-file";
		const HEADER_FILE_STR: &str = "header-file";

		// Get all matches from cli
		let matches = ClapApp::new("Dcb Debugger")
			.version("0.0")
			.author("Filipe [...] <[...]@gmail.com>")
			.about("Compiles code from assembly, runs and debugs it")
			.arg(
				ClapArg::with_name(INPUT_FILE_STR)
					.help("Sets the input file to use")
					.required(true)
					.index(1),
			)
			.arg(
				ClapArg::with_name(HEADER_FILE_STR)
					.help("Sets the header file to use")
					.long("header")
					.short("h"),
			)
			.get_matches();

		// Get the input filename
		// Note: required
		let input_path = matches
			.value_of(INPUT_FILE_STR)
			.map(PathBuf::from)
			.expect("Unable to get required argument `input-file`");

		// Get the header filename
		let header_path = match matches.value_of(HEADER_FILE_STR) {
			Some(path) => PathBuf::from(path),
			None => {
				let mut path = input_path.clone().into_os_string();
				path.push(".header");
				PathBuf::from(path)
			},
		};

		// Return the cli data
		Self {
			input_path,
			header_path,
		}
	}
}
