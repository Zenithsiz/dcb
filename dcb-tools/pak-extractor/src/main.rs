//! `.Pak` extractor

// Features
#![feature(array_value_iter)]

// Modules
mod cli;
mod logger;

// Imports
use anyhow::Context;
use dcb_io::{pak, PakFile};
use std::{
	io::{Read, Seek, SeekFrom},
	path::Path,
};


fn main() -> Result<(), anyhow::Error> {
	// Initialize the logger
	logger::init();

	// Get all data from cli
	let cli::CliData { input_file, output_dir } = cli::CliData::new();

	// Try to extract the filesystem
	self::extract_file(&input_file, &output_dir).context("Unable to extract `pak` file")?;

	Ok(())
}

/// Extracts a `.pak` file to `output_dir`.
fn extract_file(input_file: &Path, output_dir: &Path) -> Result<(), anyhow::Error> {
	// Open the file and parse a `pak` filesystem from it.
	let mut input_file = std::fs::File::open(input_file).context("Unable to open input file")?;

	let pak_fs = PakFile::from_reader(&mut input_file).context("Unable to parse file")?;

	for (idx, entry) in pak_fs.entries().iter().enumerate() {
		let extension = match entry.header().kind {
			pak::header::Kind::Unknown0 => "UN0",
			pak::header::Kind::Unknown1 => "UN1",
			pak::header::Kind::GameScript => "MSD",
			pak::header::Kind::Animation2D => "A2D",
			pak::header::Kind::Unknown2 => "UN2",
			pak::header::Kind::FileContents => "BIN",
			pak::header::Kind::AudioSeq => "SEQ",
			pak::header::Kind::AudioVh => "VH",
			pak::header::Kind::AudioVb => "VB",
		};
		let path = output_dir.join(format!("{}.{}", idx, extension));

		log::info!("{} ({} bytes)", path.display(), entry.header().size);

		// Seek the file and read it's size at most
		input_file
			.seek(SeekFrom::Start(entry.pos()))
			.with_context(|| format!("Unable to seek to file {}", path.display()))?;
		let mut input_file = input_file.by_ref().take(u64::from(entry.header().size));

		// Then create the output file and copy.
		let mut output_file = std::fs::File::create(&path).with_context(|| format!("Unable to create file {}", path.display()))?;
		std::io::copy(&mut input_file, &mut output_file).with_context(|| format!("Unable to write file {}", path.display()))?;
	}

	Ok(())
}
