//! Command line arguments

// Imports
use clap::{App as ClapApp, Arg as ClapArg};
use std::path::PathBuf;

/// Command line data
#[derive(PartialEq, Clone, Debug)]
pub struct CliData {
	/// The input file
	pub input_path: PathBuf,

	/// Output directory
	pub output_dir_path: PathBuf,

	/// If instruction positions should be printed
	pub print_inst_pos: bool,

	/// Path to print the header to
	pub header_path: Option<PathBuf>,

	/// Known data path
	pub known_data_path: PathBuf,

	/// Foreign data path
	pub foreign_data_path: PathBuf,

	/// Known functions path
	pub known_funcs_path: PathBuf,
}

impl CliData {
	/// Constructs all of the cli data given and returns it
	#[allow(clippy::new_without_default)] // No need
	pub fn new() -> Self {
		const INPUT_FILE_STR: &str = "input-file";
		const OUTPUT_DIR_STR: &str = "output-dir";
		const PRINT_INST_POS_STR: &str = "print-inst-pos";
		const PRINT_HEADER_STR: &str = "print-header";

		const KNOWN_DATA_PATH_STR: &str = "known-data-path";
		const FOREIGN_DATA_PATH_STR: &str = "foreign-data-path";
		const KNOWN_FUNCS_PATH_STR: &str = "known-funcs-path";

		// Get all matches from cli
		let matches = ClapApp::new("Dcb disassembler")
			.version("0.0")
			.author("Filipe [...] <[...]@gmail.com>")
			.about("Disassembles code")
			.arg(
				ClapArg::with_name(INPUT_FILE_STR)
					.long(INPUT_FILE_STR)
					.help("Sets the input game file to use")
					.required(true)
					.index(1)
					.takes_value(true),
			)
			// TODO: Use this argument
			.arg(
				ClapArg::with_name(OUTPUT_DIR_STR)
					.long(OUTPUT_DIR_STR)
					.short("o")
					.help("Sets the output directory to extract to")
					.takes_value(true),
			)
			.arg(
				ClapArg::with_name(PRINT_INST_POS_STR)
					.long(PRINT_INST_POS_STR)
					.help("If instructions' positions should be printed"),
			)
			.arg(
				ClapArg::with_name(PRINT_HEADER_STR)
					.long(PRINT_HEADER_STR)
					.help("Path of the header file to output to")
					.takes_value(true),
			)
			.arg(
				ClapArg::with_name(KNOWN_DATA_PATH_STR)
					.long(KNOWN_DATA_PATH_STR)
					.help("Sets the path of the known data")
					.takes_value(true),
			)
			.arg(
				ClapArg::with_name(FOREIGN_DATA_PATH_STR)
					.long(FOREIGN_DATA_PATH_STR)
					.help("Sets the path of the foreign data")
					.takes_value(true),
			)
			.arg(
				ClapArg::with_name(KNOWN_FUNCS_PATH_STR)
					.long(KNOWN_FUNCS_PATH_STR)
					.help("Sets the path of the known funcs")
					.takes_value(true),
			)
			.get_matches();

		// Get the input filename
		// Note: required
		let input_path = matches
			.value_of(INPUT_FILE_STR)
			.map(PathBuf::from)
			.expect("Unable to get required argument `input-file`");

		// Get the output directory, or just use `src`
		let output_dir_path = match matches.value_of(OUTPUT_DIR_STR) {
			Some(path) => PathBuf::from(path),
			None => PathBuf::from("src/"),
		};


		let print_inst_pos = matches.is_present(PRINT_INST_POS_STR);
		let header_path = matches.value_of(PRINT_HEADER_STR).map(PathBuf::from);

		let known_data_path = matches
			.value_of(KNOWN_DATA_PATH_STR)
			.unwrap_or(default_paths::KNOWN_DATA);
		let foreign_data_path = matches
			.value_of(FOREIGN_DATA_PATH_STR)
			.unwrap_or(default_paths::FOREIGN_DATA);
		let known_funcs_path = matches
			.value_of(KNOWN_FUNCS_PATH_STR)
			.unwrap_or(default_paths::KNOWN_FUNCS);

		// Return the cli data
		Self {
			input_path,
			output_dir_path,
			print_inst_pos,
			header_path,

			known_data_path: PathBuf::from(known_data_path),
			foreign_data_path: PathBuf::from(foreign_data_path),
			known_funcs_path: PathBuf::from(known_funcs_path),
		}
	}
}


/// Default paths
mod default_paths {
	/// Known data path
	pub const KNOWN_DATA: &str = "resources/asm/game_data.yaml";

	/// Foreign data path
	pub const FOREIGN_DATA: &str = "resources/asm/foreign_data.yaml";

	/// Known functions path
	pub const KNOWN_FUNCS: &str = "resources/asm/game_funcs.yaml";
}
