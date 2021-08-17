//! Cli manager

// Imports
use clap::{App as ClapApp, Arg as ClapArg};
use std::path::{Path, PathBuf};

/// Data from the command line
#[derive(PartialEq, Clone, Debug)]
pub struct CliData {
	/// Input file
	pub input_file: PathBuf,

	/// The output file
	pub output_file: PathBuf,
}

impl CliData {
	/// Constructs all of the cli data given and returns it
	pub fn new() -> Self {
		// Get all matches from cli
		let matches = ClapApp::new("Iso to Bin")
			.version("0.1")
			.author("Filipe [...] <[...]@gmail.com>")
			.about("Coverts an ISO 9660 filesystem file to a `.bin` file in the CdRom/XA format")
			.arg(
				ClapArg::with_name("INPUT_FILE")
					.help("The input file to use")
					.required(true)
					.index(1),
			)
			.arg(
				ClapArg::with_name("OUTPUT")
					.help("The file to output to")
					.long_help(
						"The file to output to. If not specified, a file with the input's name appended by `.bin` \
						 will be used",
					)
					.short("o")
					.long("output")
					.takes_value(true)
					.required(false),
			)
			.get_matches();

		// Get the input filename
		// Note: required
		let input_file = matches
			.value_of("INPUT_FILE")
			.map(Path::new)
			.map(Path::to_path_buf)
			.expect("Unable to get required argument `INPUT_FILE`");

		// Try to get the output, else use the input filename + `.bin`
		let output_file = match matches.value_of("OUTPUT") {
			Some(output) => PathBuf::from(output),
			None => {
				let extension = match input_file.extension() {
					Some(extension) => format!("{}.bin", extension.to_string_lossy()),
					None => "bin".to_string(),
				};

				input_file.with_extension(extension)
			},
		};

		// Return the data
		Self {
			input_file,
			output_file,
		}
	}
}
