//! Command line arguments

// Imports
use clap::{App as ClapApp, Arg as ClapArg};
use std::path::PathBuf;

/// Arguments
#[derive(PartialEq, Clone, Debug)]
pub struct Args {
	/// The input file
	pub input_path: PathBuf,

	/// Output directory
	pub output_dir_path: PathBuf,

	/// If instruction positions should be printed
	pub print_inst_pos: bool,

	/// Path to print the header to
	pub header_path: Option<PathBuf>,

	/// Game data path
	pub game_data_path: Option<PathBuf>,

	/// Foreign data path
	pub foreign_data_path: Option<PathBuf>,

	/// Game functions path
	pub game_funcs_path: Option<PathBuf>,
}

impl Args {
	/// Returns the arguments given
	#[allow(clippy::new_without_default)] // No need
	pub fn get() -> Self {
		const INPUT_FILE_STR: &str = "input-file";
		const OUTPUT_DIR_STR: &str = "output-dir";
		const PRINT_INST_POS_STR: &str = "print-inst-pos";
		const PRINT_HEADER_STR: &str = "print-header";

		const GAME_DATA_PATH_STR: &str = "game-data-path";
		const FOREIGN_DATA_PATH_STR: &str = "foreign-data-path";
		const GAME_FUNCS_PATH_STR: &str = "game-funcs-path";

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
				ClapArg::with_name(GAME_DATA_PATH_STR)
					.long(GAME_DATA_PATH_STR)
					.help("Sets the path of the game data")
					.takes_value(true),
			)
			.arg(
				ClapArg::with_name(FOREIGN_DATA_PATH_STR)
					.long(FOREIGN_DATA_PATH_STR)
					.help("Sets the path of the foreign data")
					.takes_value(true),
			)
			.arg(
				ClapArg::with_name(GAME_FUNCS_PATH_STR)
					.long(GAME_FUNCS_PATH_STR)
					.help("Sets the path of the game funcs")
					.takes_value(true),
			)
			.get_matches();

		// Get the input filename
		// Note: required
		let input_path = matches
			.value_of(INPUT_FILE_STR)
			.map(PathBuf::from)
			.expect("Unable to get required argument `input-file`");

		// Get the output directory, or just use the default
		let output_dir_path = match matches.value_of(OUTPUT_DIR_STR) {
			Some(path) => PathBuf::from(path),
			None => PathBuf::from("game/asm/"),
		};


		let print_inst_pos = matches.is_present(PRINT_INST_POS_STR);
		let header_path = matches.value_of(PRINT_HEADER_STR).map(PathBuf::from);

		let game_data_path = matches.value_of(GAME_DATA_PATH_STR).map(PathBuf::from);
		let foreign_data_path = matches.value_of(FOREIGN_DATA_PATH_STR).map(PathBuf::from);
		let game_funcs_path = matches.value_of(GAME_FUNCS_PATH_STR).map(PathBuf::from);

		// Return the cli data
		Self {
			input_path,
			output_dir_path,
			print_inst_pos,
			header_path,
			game_data_path,
			foreign_data_path,
			game_funcs_path,
		}
	}
}
