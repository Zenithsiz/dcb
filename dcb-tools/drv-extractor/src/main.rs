//! `.DRV` extractor

// Features
#![feature(array_value_iter)]

// Modules
mod cli;

// Imports
use anyhow::Context;
use dcb_io::{
	drv::{dir::entry::DirEntryReaderKind, DirReader},
	DrvFsReader,
};
use std::{fs, io, path::Path};


fn main() -> Result<(), anyhow::Error> {
	// Initialize the logger
	simplelog::TermLogger::init(log::LevelFilter::Info, simplelog::Config::default(), simplelog::TerminalMode::Stderr)
		.expect("Unable to initialize logger");

	// Get all data from cli
	let cli::CliData { input_file, output_dir } = cli::CliData::new();

	// Then try to extract the filesystem
	self::extract_file(&input_file, &output_dir).with_context(|| format!("Unable to extract {}", input_file.display()))?;

	Ok(())
}

/// Extracts a `.drv` file to `output_dir`.
fn extract_file(input_file: &Path, output_dir: &Path) -> Result<(), anyhow::Error> {
	// Open the file and parse a `drv` filesystem from it.
	let mut input_file = fs::File::open(input_file).context("Unable to open input file")?;

	// Create output directory if it doesn't exist
	self::try_create_folder(output_dir)?;

	// Then extract the tree
	self::extract_tree(&mut input_file, &DrvFsReader::root(), output_dir).context("Unable to extract files from root")
}

/// Extracts a `.drv` file from a reader and starting directory
fn extract_tree<R: io::Read + io::Seek>(reader: &mut R, dir: &DirReader, path: &Path) -> Result<(), anyhow::Error> {
	// Get all entries
	// Note: We need to collect to free the reader so it can seek to the next files.
	let entries: Vec<_> = dir
		.read_entries(reader)
		.with_context(|| format!("Unable to get directory entries of {}", path.display()))?
		.collect();

	// Then extract each entry
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

		// Then check it's type
		match entry.kind() {
			// If it's a file, create the file and write all contents
			DirEntryReaderKind::File(file) => {
				log::info!("Extracting {} ({} bytes)", path.display(), file.size());

				// Get the file's reader.
				let mut reader = file.reader(reader).with_context(|| format!("Unable to read file {}", path.display()))?;

				// Then create the output file and copy.
				let mut output_file = fs::File::create(&path).with_context(|| format!("Unable to create file {}", path.display()))?;
				std::io::copy(&mut reader, &mut output_file).with_context(|| format!("Unable to write file {}", path.display()))?;

				// And set the file's modification time
				if let Err(err) = filetime::set_file_handle_times(&output_file, None, Some(time)) {
					log::warn!(
						"Unable to write date for file {}: {}",
						path.display(),
						dcb_util::DisplayWrapper::new(|f| dcb_util::fmt_err(&err, f))
					);
				}
			},

			// If it's a directory, create it and recurse for all it's entries
			DirEntryReaderKind::Dir(dir) => {
				log::info!("Extracting {}", path.display());

				// Create the directory and set it's modification date
				self::try_create_folder(&path).with_context(|| format!("Unable to create directory {}", path.display()))?;
				if let Err(err) = filetime::set_file_mtime(&path, time) {
					log::warn!(
						"Unable to write date for directory {}: {}",
						path.display(),
						dcb_util::DisplayWrapper::new(|f| dcb_util::fmt_err(&err, f))
					);
				}

				// Then recurse over it
				self::extract_tree(reader, dir, &path).with_context(|| format!("Unable to extract directory {}", path.display()))?;
			},
		}
	}

	Ok(())
}

/// Attempts to create a folder. Returns `Ok` if it already exists.
#[allow(clippy::create_dir)] // We only want to create a single level
fn try_create_folder(path: impl AsRef<std::path::Path>) -> Result<(), io::Error> {
	match fs::create_dir(&path) {
		// If we managed to create it, or it already exists, return `Ok`
		Ok(_) => Ok(()),
		Err(err) if err.kind() == io::ErrorKind::AlreadyExists => Ok(()),
		Err(err) => Err(err),
	}
}
