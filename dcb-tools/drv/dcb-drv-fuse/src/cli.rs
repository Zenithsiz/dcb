//! Cli manager

// Imports
use clap::{App as ClapApp, Arg as ClapArg};
use std::path::PathBuf;

/// Data from the command line
#[derive(PartialEq, Clone, Debug)]
pub struct CliData {
	/// Input file
	pub input_file: PathBuf,

	/// Mount point
	pub mount_point: PathBuf,
}

impl CliData {
	/// Constructs all of the cli data given and returns it
	pub fn new() -> Self {
		// Get all matches from cli
		let matches = ClapApp::new("Drv Fuse")
			.version("0.1")
			.author("Filipe [...] <[...]@gmail.com>")
			.about("Mounts a `.drv` file as a `fuse` filesystem")
			.arg(
				ClapArg::with_name("INPUT_FILE")
					.help("The input file to use")
					.required(true)
					.takes_value(true)
					.index(1),
			)
			.arg(
				ClapArg::with_name("MOUNT_POINT")
					.help("The mount point")
					.long("mount-point")
					.required(true)
					.takes_value(true)
					.index(2),
			)
			.get_matches();

		// Get the input filename
		// Note: required
		let input_file = matches
			.value_of("INPUT_FILE")
			.map(PathBuf::from)
			.expect("Unable to get required argument");
		let mount_point = matches
			.value_of("MOUNT_POINT")
			.map(PathBuf::from)
			.expect("Unable to get required argument");

		// Return the data
		Self {
			input_file,
			mount_point,
		}
	}
}
