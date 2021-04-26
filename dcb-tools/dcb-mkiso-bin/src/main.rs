//! Iso extractor from `.bin` files.

// Features
#![feature(unwrap_infallible)]

// Modules
mod cli;

// Imports
use anyhow::Context;
use cli::CliData;
use std::{fs, path::PathBuf};

fn main() -> Result<(), anyhow::Error> {
	// Initialize the logger
	simplelog::TermLogger::init(
		log::LevelFilter::Info,
		simplelog::Config::default(),
		simplelog::TerminalMode::Stderr,
	)
	.expect("Unable to initialize logger");

	// Get all data from cli
	let cli_data = CliData::new();

	// If we don't have an output, use the input filename with `.iso`
	let output_file = match &cli_data.output_file {
		Some(output) => output.to_path_buf(),
		None => {
			let mut path = cli_data.input_dir.as_os_str().to_os_string();
			path.push(".iso");
			PathBuf::from(path)
		},
	};

	// Read the header file
	let header_file_path = {
		let mut path = cli_data.input_dir.as_os_str().to_os_string();
		path.push(".header");
		PathBuf::from(path)
	};
	let mut header_file = fs::File::create(header_file_path).context("Unable to create output header file")?;
	let _header: Header = serde_yaml::from_reader(&mut header_file).context("Unable to read header")?;

	// Create the output file
	let _output_file = fs::File::create(output_file).context("Unable to create output file")?;

	// Go through all files in the directory
	for entry in std::fs::read_dir(&cli_data.input_dir).context("Unable to read input directory")? {
		// Get the entry
		let entry = entry.context("Unable to read entry")?;
		let path = entry.path();
		let metadata = entry.metadata().context("Unable to get file metadata")?;

		// If it's a directory, skip
		if metadata.is_dir() {
			log::warn!("Skipping directory: {}", path.display());
			continue;
		}

		// Open the input file and write it
		let _input_file = fs::File::open(&path).context("Unable to open input file")?;

		todo!();
	}

	Ok(())
}

/// Header
#[derive(PartialEq, Eq, Clone, Debug)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct Header {
	/// System Id
	pub system_id: String,

	/// Volume Id
	pub volume_id: String,

	/// Volume space size
	pub volume_space_size: u32,

	/// Volume sequence_number
	pub volume_sequence_number: u16,

	/// Logical block size
	pub logical_block_size: u16,

	/// Volume set identifier
	pub volume_set_id: String,

	/// Publisher identifier
	pub publisher_id: String,

	/// Data preparer identifier
	pub data_preparer_id: String,

	/// Application identifier
	pub application_id: String,

	/// Copyright file identifier
	pub copyright_file_id: String,

	/// Abstract file identifier
	pub abstract_file_id: String,

	/// Bibliographic file identifier
	pub bibliographic_file_id: String,

	/// Volume creation date time
	pub volume_creation_date_time: String,

	/// Volume modification date time
	pub volume_modification_date_time: String,

	/// Volume expiration date time
	pub volume_expiration_date_time: String,

	/// Volume effective date time
	pub volume_effective_date_time: String,
}
