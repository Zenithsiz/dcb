//! Cli manager

// Imports
use clap::{App as ClapApp, AppSettings, Arg as ClapArg};
use std::path::PathBuf;

/// Data from the command line
#[derive(PartialEq, Clone, Debug)]
pub struct CliData {
	/// Input dir
	pub input_dir: PathBuf,

	/// The output file
	pub output_file: Option<PathBuf>,
}

impl CliData {
	/// Constructs all of the cli data given and returns it
	pub fn new() -> Self {
		const INPUT_DIR_STR: &str = "input-dir";
		const OUTPUT_FILE_STR: &str = "output-file";

		// Get all matches from cli
		let matches = ClapApp::new("Iso Extractor")
			.version("0.1")
			.author("Filipe Rodrigues <filipejacintorodrigues1@gmail.com>")
			.about("Packer for `bin` files.")
			.setting(AppSettings::ArgRequiredElseHelp)
			.arg(
				ClapArg::with_name(INPUT_DIR_STR)
					.help("The input directory to use")
					.required(true)
					.multiple(true),
			)
			.arg(
				ClapArg::with_name(OUTPUT_FILE_STR)
					.help("The file to output to")
					.short("o")
					.long("output-file")
					.takes_value(true),
			)
			.get_matches();

		// Get the input filename
		// Note: required
		let input_dir = matches
			.value_of(INPUT_DIR_STR)
			.map(PathBuf::from)
			.expect("Unable to get required argument `input-dir`");


		let output_file = matches.value_of(OUTPUT_FILE_STR).map(PathBuf::from);

		// Return the data
		Self { input_dir, output_file }
	}
}
