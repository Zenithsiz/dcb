//! `.DRV` extractor

// Features
#![feature(array_value_iter)]

// Modules
mod cli;
mod logger;

// Imports
use anyhow::Context;
use dcb_io::{
	drv::{dir::entry::DirEntryReaderKind, DirReader},
	DrvFsReader,
};
use std::{io, path::Path};


fn main() -> Result<(), anyhow::Error> {
	// Initialize the logger
	logger::init();

	// Get all data from cli
	let cli::CliData { input_file, output_dir } = cli::CliData::new();

	// Try to extract the filesystem
	self::extract_file(&input_file, &output_dir).context("Unable to extract `drv` file")?;

	Ok(())
}

/// Extracts a `.drv` file to `output_dir`.
fn extract_file(input_file: &Path, output_dir: &Path) -> Result<(), anyhow::Error> {
	// Open the file and parse a `drv` filesystem from it.
	let mut input_file = std::fs::File::open(input_file).context("Unable to open input file")?;

	// Create output directory if it doesn't exist
	self::try_create_folder(output_dir)?;

	// Then extract the tree
	self::extract_tree(&mut input_file, &DrvFsReader::root(), output_dir).context("Unable to extract files from root")
}

/// Extracts a `.drv` file from a reader and starting directory
fn extract_tree<R: io::Read + io::Seek>(reader: &mut R, dir: &DirReader, path: &Path) -> Result<(), anyhow::Error> {
	// Then for each entry create it
	let entries: Vec<_> = dir
		.read_entries(reader)
		.with_context(|| format!("Unable to get directory entries of {}", path.display()))?
		.collect();
	for entry in entries {
		// If we can't read it, return Err
		let entry = entry.with_context(|| format!("Unable to read directory entry of {}", path.display()))?;

		// Get the filename and new path
		let name = match entry.kind() {
			DirEntryReaderKind::File(file) => format!("{}.{}", entry.name(), file.extension()),
			DirEntryReaderKind::Dir(_) => entry.name().to_string(),
		};
		let path = path.join(name);

		// Create the date
		// Note: `.DRV` only supports second precision.
		let time = filetime::FileTime::from_unix_time(entry.date().timestamp(), 0);

		// Then check what we need to do with it
		match entry.kind() {
			DirEntryReaderKind::File(file) => {
				log::info!("{} ({} bytes)", path.display(), file.size());

				// Limit the input file to it's size
				let mut reader = file.reader(reader).with_context(|| format!("Unable to read file {}", path.display()))?;

				// Then create the output file and copy.
				let mut output_file = std::fs::File::create(&path).with_context(|| format!("Unable to create file {}", path.display()))?;
				std::io::copy(&mut reader, &mut output_file).with_context(|| format!("Unable to write file {}", path.display()))?;

				// And set the file's modification time
				filetime::set_file_handle_times(&output_file, None, Some(time))
					.with_context(|| format!("Unable to write date for file {}", path.display()))?;
			},
			DirEntryReaderKind::Dir(dir) => {
				log::info!("{}", path.display());

				// Create the directory and set it's modification date
				self::try_create_folder(&path)?;
				filetime::set_file_mtime(&path, time).with_context(|| format!("Unable to write date for directory {}", path.display()))?;

				// Then recurse over it
				self::extract_tree(reader, dir, &path).with_context(|| format!("Unable to extract directory {}", path.display()))?;
			},
		}
	}

	Ok(())
}

/// Attempts to create a folder. Returns `Ok` if it already exists.
#[allow(clippy::create_dir)] // We only want to create a single level
fn try_create_folder(path: impl AsRef<std::path::Path>) -> Result<(), anyhow::Error> {
	match std::fs::create_dir(&path) {
		// If it already exists, ignore
		Ok(_) => Ok(()),
		Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => Ok(()),
		Err(err) => Err(err).with_context(|| format!("Unable to create directory {}", path.as_ref().display())),
	}
}
