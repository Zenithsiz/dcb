//! Cli manager

// Imports
use clap::{App as ClapApp, AppSettings, Arg as ClapArg};
use std::path::PathBuf;

/// Data from the command line
#[derive(PartialEq, Clone, Debug)]
pub struct CliData {
	/// Input file
	pub input_file: PathBuf,

	/// The output directory
	pub output_dir: Option<PathBuf>,
}

impl CliData {
	/// Constructs all of the cli data given and returns it
	pub fn new() -> Self {
		const INPUT_FILE_STR: &str = "input-file";
		const OUTPUT_DIR_STR: &str = "output-dir";

		// Get all matches from cli
		let matches = ClapApp::new("Iso Extractor")
			.version("0.1")
			.author("Filipe Rodrigues <filipejacintorodrigues1@gmail.com>")
			.about("Extractor for `iso` files.")
			.setting(AppSettings::ArgRequiredElseHelp)
			.arg(
				ClapArg::with_name(INPUT_FILE_STR)
					.help("The input file to use")
					.required(true)
					.multiple(true),
			)
			.arg(
				ClapArg::with_name(OUTPUT_DIR_STR)
					.help("The directory to output to")
					.long_help(
						"The directory to output to. If not specified, the parent of the input file is used. If it doesn't exist, the current \
						 directory is used",
					)
					.short("o")
					.long("output-dir")
					.takes_value(true),
			)
			.get_matches();

		// Get the input filename
		// Note: required
		let input_file = matches
			.value_of(INPUT_FILE_STR)
			.map(PathBuf::from)
			.expect("Unable to get required argument `input-file`");


		let output_dir = matches.value_of(OUTPUT_DIR_STR).map(PathBuf::from);

		// Return the data
		Self { input_file, output_dir }
	}
}
