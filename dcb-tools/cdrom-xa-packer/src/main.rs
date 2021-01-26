//! `.DRV` packer

// Features
#![feature()]

// Modules
mod cli;

// Imports
use anyhow::Context;
use dcb_cdrom_xa::{CdRomWriter, Sector};
use std::{
	fs,
	io::{self, Read},
	path::Path,
};


fn main() -> Result<(), anyhow::Error> {
	// Initialize the logger
	simplelog::TermLogger::init(log::LevelFilter::Info, simplelog::Config::default(), simplelog::TerminalMode::Stderr)
		.expect("Unable to initialize logger");

	// Get all data from cli
	let cli::CliData { input_file, output_file } = cli::CliData::new();

	// Try to pack it into a `CdRom/XA`
	self::pack_cdrom_xa(&input_file, &output_file).context("Unable to pack file")?;

	Ok(())
}

/// Packs a file into a `CdRom/XA`
fn pack_cdrom_xa(input_file: &Path, output_file: &Path) -> Result<(), anyhow::Error> {
	// Open the input file
	let mut input_file = fs::File::open(input_file).context("Unable to open input file")?;

	// Create the output file
	let output_file = fs::File::create(output_file).context("Unable to create output file")?;
	let mut output_file = CdRomWriter::new(output_file);

	// Read the input file by chunks of 2048.
	'write_loop: for sector_pos in 0.. {
		let mut data = [0; 2048];

		// Inlined from `Read::read_exact`.
		{
			let mut buf: &mut [u8] = &mut data;
			while !buf.is_empty() {
				match input_file.read(buf) {
					// If we get eof, check if we read anything so far.
					Ok(0) => match buf.len() {
						2048 => break 'write_loop,
						_ => break,
					},
					// If we managed to read, update our buffer.
					Ok(n) => buf = &mut buf[n..],
					Err(err) if err.kind() == io::ErrorKind::Interrupted => (),
					Err(err) => return Err(err).context("Unable to read from input file"),
				}
			}
		}

		let sector = Sector::new(data, sector_pos).context("Unable to create sector")?;

		output_file.write_sector(&sector).context("Unable to write sector to output file")?;
	}

	Ok(())
}
