//! Iso extractor from `.bin` files.

// Features
#![feature(unwrap_infallible)]

// Modules
mod cli;

// Imports
use anyhow::Context;
use cli::CliData;
use dcb_bytes::Bytes;
use dcb_cdrom_xa::CdRomReader;
use dcb_iso9660::{date_time::DecDateTime, FilesystemReader};
use std::{fs, io, path::PathBuf};

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

	// If we don't have an output, try the input filename without extension if it's `.iso`, else use `.`
	let output_dir = match &cli_data.output_dir {
		Some(output) => output.to_path_buf(),
		None => match cli_data.input_file.extension() {
			Some(extension) if extension.eq_ignore_ascii_case("iso") => cli_data.input_file.with_extension(""),
			_ => PathBuf::from("."),
		},
	};

	// Create output directory if it doesn't exist
	dcb_util::try_create_folder(&output_dir)
		.with_context(|| format!("Unable to create directory {}", output_dir.display()))?;

	// Open the file.
	let input_file = fs::File::open(&cli_data.input_file).context("Unable to open input file")?;
	let input_file = io::BufReader::new(input_file);
	let mut input_file = CdRomReader::new(input_file);
	let fs_reader = FilesystemReader::new(&mut input_file).context("Unable to create filesystem reader")?;

	// Create the header and output it
	let header_file_path = {
		let mut path = cli_data.input_file.as_os_str().to_os_string();
		path.push(".header");
		PathBuf::from(path)
	};
	let mut header_file = fs::File::create(header_file_path).context("Unable to create output header file")?;
	{
		let date_time_to_string = |date_time: DecDateTime| {
			let mut bytes = [0; 0x11];
			date_time.serialize_bytes(&mut bytes).into_ok();
			std::str::from_utf8(&bytes)
				.expect("Date time was invalid utf8")
				.to_owned()
		};

		let volume = fs_reader.primary_volume_descriptor();
		let header = Header {
			system_id:                     volume.system_id.as_lossy_str().to_string(),
			volume_id:                     volume.volume_id.as_lossy_str().to_string(),
			volume_space_size:             volume.volume_space_size,
			volume_sequence_number:        volume.volume_sequence_number,
			logical_block_size:            volume.logical_block_size,
			volume_set_id:                 volume.volume_set_id.as_lossy_str().to_string(),
			publisher_id:                  volume.publisher_id.as_lossy_str().to_string(),
			data_preparer_id:              volume.data_preparer_id.as_lossy_str().to_string(),
			application_id:                volume.application_id.as_lossy_str().to_string(),
			copyright_file_id:             volume.copyright_file_id.as_lossy_str().to_string(),
			abstract_file_id:              volume.abstract_file_id.as_lossy_str().to_string(),
			bibliographic_file_id:         volume.bibliographic_file_id.as_lossy_str().to_string(),
			volume_creation_date_time:     date_time_to_string(volume.volume_creation_date_time),
			volume_modification_date_time: date_time_to_string(volume.volume_modification_date_time),
			volume_expiration_date_time:   date_time_to_string(volume.volume_expiration_date_time),
			volume_effective_date_time:    date_time_to_string(volume.volume_effective_date_time),
		};
		serde_yaml::to_writer(&mut header_file, &header).context("Unable to write header")?;
	}

	// Extract all files in the root directory
	let root_dir = fs_reader
		.root_dir()
		.read_dir(&mut input_file)
		.context("Unable to read root directory entry")?;

	for entry in root_dir.entries() {
		let name = entry.name.without_version();

		if entry.is_dir() {
			log::warn!("Skipping directory in root dir: {}", &name);
			continue;
		}

		// Else extract it
		let mut file = entry.read_file(&mut input_file).context("Unable to read file")?;

		// Open the output file
		let mut output_file = fs::File::create(output_dir.join(name)).context("Unable to open output file")?;

		// And copy the file
		std::io::copy(&mut file, &mut output_file).context("Unable to write output file")?;
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
