//! Cli manager for the extractor

// Filesystem
use std::path::{Path, PathBuf};

// Clap
use clap::{Arg as ClapArg, App as ClapApp};

// Errors
use err_panic::ErrorExtPanic;


/// Data from the command line
#[derive(PartialEq, Clone, Debug)]
pub struct CliData
{
	/// The game file
	pub game_file_path: PathBuf,
	
	/// The input directory
	pub input_dir: PathBuf,
}

impl CliData
{
	/// Constructs all of the cli data given and returns it
	pub fn new() -> Self
	{
		// Get all matches from cli
		let matches = ClapApp::new("Dcb Patcher")
			.version("0.0")
			.author("Filipe [...] <[...]@gmail.com>")
			.about("Patches data to a Digimon Digital Card Battle `.bin` game file")
			.arg( ClapArg::with_name("GAME_FILE")
				.help("Sets the game file to use")
				.required(true)
				.index(1)
			)
			.arg( ClapArg::with_name("INPUT")
				.help("Sets the output directory to use")
				.short("i")
				.long("input")
				.index(2)
				.takes_value(true)
				.required(true)
			)
			.get_matches();
		
		// Get the ouput filename
		// Note: required
		let game_file_path = matches.value_of("GAME_FILE")
			.map(Path::new)
			.map(Path::to_path_buf)
			.panic_msg("Unable to get required argument `GAME_FILE`");
		
		// Get the input dir
		// Note: required
		let input_dir = matches.value_of("INPUT")
			.map(Path::new)
			.map(Path::to_path_buf)
			.panic_msg("Unable to get required argument `INPUT`");
		
		// Return the cli data
		Self {
			game_file_path,
			input_dir,
		}
	}
}
