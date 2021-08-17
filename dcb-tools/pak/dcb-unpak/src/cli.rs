//! Cli manager

// Imports
use clap::{App as ClapApp, Arg as ClapArg};
use std::path::PathBuf;

/// Data from the command line
#[derive(PartialEq, Clone, Debug)]
pub struct CliData {
	/// Input files
	pub input_files: Vec<PathBuf>,

	/// The output directory
	pub output_dir: Option<PathBuf>,

	pub quiet: bool,

	pub warn_on_override: bool,

	pub only_list: bool,
}

impl CliData {
	/// Constructs all of the cli data given and returns it
	pub fn new() -> Self {
		const INPUT_FILES_STR: &str = "input-files";
		const OUTPUT_DIR_STR: &str = "output-dir";
		const QUIET_STR: &str = "quiet";
		const WARN_ON_OVERRIDE_STR: &str = "warn-on-override";
		const ONLY_LIST: &str = "only-list";

		// Get all matches from cli
		let matches = ClapApp::new("Pak Extractor")
			.version("0.0")
			.author("Filipe Rodrigues <filipejacintorodrigues1@gmail.com>")
			.about("Extracts the filesystem in a `.pak` filesystem")
			.arg(
				ClapArg::with_name(INPUT_FILES_STR)
					.help("The input files to use")
					.required(true)
					.multiple(true),
			)
			.arg(
				ClapArg::with_name(OUTPUT_DIR_STR)
					.help("The directory to output to")
					.long_help(
						"The directory to output to. If not specified, the parent of the input file is used. If it \
						 doesn't exist, the current directory is used",
					)
					.short("o")
					.long("output-dir")
					.takes_value(true),
			)
			.arg(
				ClapArg::with_name(QUIET_STR)
					.help("Suppresses listing of extracted files")
					.long_help(
						"Suppresses printing on `stdout` of this program. If any errors or warnings occur, they will \
						 still be printed to stderr",
					)
					.short("q")
					.long("quiet"),
			)
			.arg(
				ClapArg::with_name(WARN_ON_OVERRIDE_STR)
					.help("Warns when overriding files that already exist.")
					.long_help(
						"Warns if this program would override existing files. By default no warnings are produced.",
					)
					.long("warn-on-override"),
			)
			.arg(
				ClapArg::with_name(ONLY_LIST)
					.help("Only lists contents, does not extract")
					.long_help("Lists only the contents within the pak file, doesn't create any files.")
					.short("l")
					.long("only-list"),
			)
			.get_matches();

		// Get the input filename
		// Note: required
		let input_files: Vec<_> = matches
			.values_of(INPUT_FILES_STR)
			.expect("Unable to get required argument `input-file`")
			.map(PathBuf::from)
			.collect();


		let output_dir = matches.value_of(OUTPUT_DIR_STR).map(PathBuf::from);

		let quiet = matches.is_present(QUIET_STR);

		let warn_on_override = matches.is_present(WARN_ON_OVERRIDE_STR);

		let only_list = matches.is_present(ONLY_LIST);

		// Return the data
		Self {
			input_files,
			output_dir,
			quiet,
			warn_on_override,
			only_list,
		}
	}
}
