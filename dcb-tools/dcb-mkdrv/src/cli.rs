//! Cli manager

// Imports
use clap::{App as ClapApp, Arg as ClapArg};
use std::path::{Path, PathBuf};

/// Data from the command line
#[derive(PartialEq, Clone, Debug)]
pub struct CliData {
	/// Input directory
	pub input_dir: PathBuf,

	/// The output file
	pub output_file: PathBuf,
}

impl CliData {
	/// Constructs all of the cli data given and returns it
	pub fn new() -> Self {
		// Get all matches from cli
		let matches = ClapApp::new("Drv Packer")
			.version("0.1")
			.author("Filipe [...] <[...]@gmail.com>")
			.about("Packs a folder into a `.drv` file")
			.arg(ClapArg::with_name("INPUT_DIR").help("The input directory to use").required(true).index(1))
			.arg(
				ClapArg::with_name("OUTPUT")
					.help("The file to output to")
					.long_help("The file to output to. If not specified, a file with the directory's name appended by `.drv` will be used")
					.short("o")
					.long("output")
					.takes_value(true)
					.required(false),
			)
			.get_matches();

		// Get the input filename
		// Note: required
		let input_dir = matches
			.value_of("INPUT_DIR")
			.map(Path::new)
			.map(Path::to_path_buf)
			.expect("Unable to get required argument `INPUT_DIR`");

		// Try to get the output, else use the input filename + `.drv`
		let output_file = match matches.value_of("OUTPUT") {
			Some(output) => PathBuf::from(output),
			None => {
				let extension = match input_dir.extension() {
					Some(extension) => format!("{}.drv", extension.to_string_lossy()),
					None => "drv".to_string(),
				};

				input_dir.with_extension(extension)
			},
		};

		// Return the data
		Self { input_dir, output_file }
	}
}
