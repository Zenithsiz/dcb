//! Cli manager for the extractor

// Filesystem
use std::path::{Path, PathBuf};

// Clap
use clap::{Arg as ClapArg, App as ClapApp};

// Errors
use err_panic::ErrorExtPanic;


/// All of the data received form the command line
/// 
/// # Public fields
/// All fields are public as this type has no invariants.
pub struct CliData
{
	/// The input filename
	pub input_filename: PathBuf,
	
	/// The output directory
	pub output_dir: PathBuf,
}

impl CliData
{
	/// Constructs all of the cli data given and returns it
	pub fn new() -> Self
	{
		// Get all matches from cli
		let matches = ClapApp::new("Dcb Extractor")
			.version("0.0")
			.author("Filipe [...] <[...]@gmail.com>")
			.about("Extracts all data from a Digimon Digital Card Battle `.bin` game file")
			.arg( ClapArg::with_name("INPUT")
				.help("Sets the input game file to use")
				.required(true)
				.index(1)
			)
			.arg( ClapArg::with_name("OUTPUT")
				.help("Sets the output directory to use")
				.short("o")
				.long("output")
				.takes_value(true)
				.required(false)
			)
			.get_matches();
		
		// Get the input filename
		// Note: required
		let input_filename = matches.value_of("INPUT")
			.map(Path::new)
			.map(Path::to_path_buf)
			.panic_msg("Unable to get required argument `INPUT`");
		
		// Try to get the output
		let output_dir = match matches.value_of("OUTPUT") {
			Some(output) => PathBuf::from(output),
			None => input_filename
				.parent()
				.unwrap_or_else(|| Path::new("."))
				.to_path_buf()
		};
		
		// Return the cli data
		Self {
			input_filename,
			output_dir,
		}
	}
}
