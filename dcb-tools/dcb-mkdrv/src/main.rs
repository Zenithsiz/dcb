//! `.DRV` packer

// Features
#![feature(seek_stream_len, min_type_alias_impl_trait)]

// Modules
mod cli;
mod dir_lister;

// Imports
use anyhow::Context;
use dcb_drv::DrvFsWriter;
use std::{fs, path::Path};


fn main() -> Result<(), anyhow::Error> {
	// Initialize the logger
	simplelog::TermLogger::init(
		log::LevelFilter::Info,
		simplelog::Config::default(),
		simplelog::TerminalMode::Stderr,
	)
	.expect("Unable to initialize logger");

	// Get all data from cli
	let cli::CliData { input_dir, output_file } = cli::CliData::new();

	// Try to pack the filesystem
	self::pack_filesystem(&input_dir, &output_file).context("Unable to pack `drv` file")?;

	Ok(())
}

/// Extracts a `.drv` file to `output_dir`.
fn pack_filesystem(input_dir: &Path, output_file: &Path) -> Result<(), anyhow::Error> {
	// Create the output file
	let mut output_file = fs::File::create(output_file).context("Unable to create output file")?;

	// Create the filesystem writer
	let root_entries =
		dir_lister::DirLister::new(input_dir).context("Unable to create new dir lister for root directory")?;
	DrvFsWriter::write_fs(&mut output_file, root_entries).context("Unable to write filesystem")?;

	// Then pad the file to a sector `2048` if it isn't already
	let len = output_file
		.metadata()
		.context("Unable to get output file metadata")?
		.len();
	if len % 2048 != 0 {
		output_file
			.set_len(2048 * ((len + 2047) / 2048))
			.context("Unable to set file length")?;
	}

	Ok(())
}
