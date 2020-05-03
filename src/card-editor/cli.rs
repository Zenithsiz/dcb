//! Cli manager for the extractor

// Filesystem
use std::path::{Path, PathBuf};

// Clap
use clap::{App as ClapApp, Arg as ClapArg};

/// Data from the command line
#[derive(PartialEq, Clone, Debug)]
pub struct CliData {
	/// The data directory
	pub data_dir: PathBuf,
}

impl CliData {
	/// Constructs all of the cli data given and returns it
	pub fn new() -> Self {
		// Get all matches from cli
		let matches = ClapApp::new("Dcb Card Editor")
			.version("0.0")
			.author("Filipe [...] <[...]@gmail.com>")
			.about("Edits card data from Digimon Digital Card Battle extracted files")
			.arg(
				ClapArg::with_name("INPUT")
					.help("Sets the Data directory to use")
					.short("i")
					.long("input")
					.index(1)
					.takes_value(true)
					.required(true),
			)
			.get_matches();

		// Get the data dir as either an input or the current directory
		let data_dir = matches.value_of("INPUT").map_or_else(|| Path::new("."), Path::new).to_path_buf();

		// Return the cli data
		Self { data_dir }
	}
}
