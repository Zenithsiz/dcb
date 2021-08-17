//! CdRom packer

// Modules
mod cli;

// Imports
use anyhow::Context;
use dcb_cdrom_xa::CdRomReader;
use std::{fs, io::Write, path::Path};


fn main() -> Result<(), anyhow::Error> {
	// Initialize the logger
	simplelog::TermLogger::init(
		log::LevelFilter::Info,
		simplelog::Config::default(),
		simplelog::TerminalMode::Stderr,
	)
	.expect("Unable to initialize logger");

	// Get all data from cli
	let cli::CliData {
		input_file,
		output_file,
	} = cli::CliData::new();

	// Try to extract it into a `iso`
	self::extract_cdrom_xa(&input_file, &output_file).context("Unable to extract file")?;

	Ok(())
}

/// Packs a file into a `CdRom/XA`
fn extract_cdrom_xa(input_file: &Path, output_file: &Path) -> Result<(), anyhow::Error> {
	// Open the input file
	let input_file = fs::File::open(input_file).context("Unable to open input file")?;
	let mut input_file = CdRomReader::new(input_file);

	// Create the output file
	let mut output_file = fs::File::create(output_file).context("Unable to create output file")?;

	// Read all sectors
	for sector in input_file.read_sectors() {
		let sector = sector.context("Unable to read sector")?;

		output_file
			.write_all(sector.data.as_ref())
			.context("Unable to write data")?;
	}

	Ok(())
}
