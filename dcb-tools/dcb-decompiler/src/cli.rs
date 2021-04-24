//! Command line arguments

// Imports
use clap::{App as ClapApp, Arg as ClapArg};
use std::path::{Path, PathBuf};

/// Command line data
#[derive(PartialEq, Clone, Debug)]
pub struct CliData {
	/// The input file
	pub input_path: PathBuf,

	/// Output directory
	pub output_dir_path: PathBuf,

	/// If instruction positions should be printed
	pub print_inst_pos: bool,

	/// If the header should be printed
	pub print_header: bool,

	/// If the data table should be printed
	pub print_data_table: bool,
}

impl CliData {
	/// Constructs all of the cli data given and returns it
	pub fn new() -> Self {
		const INPUT_FILE_STR: &str = "input-file";
		const OUTPUT_DIR_STR: &str = "output-dir";
		const PRINT_INST_POS_STR: &str = "print-inst-pos";
		const PRINT_HEADER_STR: &str = "print-header";
		const PRINT_DATA_TABLE_STR: &str = "print-data-table";

		// Get all matches from cli
		let matches = ClapApp::new("Dcb Decompiler")
			.version("0.0")
			.author("Filipe [...] <[...]@gmail.com>")
			.about("Decompiles all code from the Digimon Digital Card Battle `.bin` game file")
			.arg(
				ClapArg::with_name(INPUT_FILE_STR)
					.help("Sets the input game file to use")
					.required(true)
					.index(1),
			)
			.arg(
				ClapArg::with_name(OUTPUT_DIR_STR)
					.long("output")
					.short("o")
					.help("Sets the input game file to use")
					.takes_value(true),
			)
			.arg(
				ClapArg::with_name(PRINT_INST_POS_STR)
					.long("print-inst-pos")
					.help("If instructions' positions should be printed"),
			)
			.arg(
				ClapArg::with_name(PRINT_HEADER_STR)
					.long("print-header")
					.help("If the header of the executable should be printed"),
			)
			.arg(
				ClapArg::with_name(PRINT_DATA_TABLE_STR)
					.long("print-data-table")
					.help("If the data table of the executable should be printed"),
			)
			.get_matches();

		// Get the input filename
		// Note: required
		let input_path = matches
			.value_of(INPUT_FILE_STR)
			.map(Path::new)
			.map(Path::to_path_buf)
			.expect("Unable to get required argument `input-file`");

		// Get the output directory, or just use `src`
		let output_dir_path = match matches.value_of(OUTPUT_DIR_STR) {
			Some(path) => PathBuf::from(path),
			None => PathBuf::from("src/"),
		};


		let print_inst_pos = matches.is_present(PRINT_INST_POS_STR);
		let print_header = matches.is_present(PRINT_HEADER_STR);
		let print_data_table = matches.is_present(PRINT_DATA_TABLE_STR);

		// Return the cli data
		Self {
			input_path,
			output_dir_path,
			print_inst_pos,
			print_header,
			print_data_table,
		}
	}
}
