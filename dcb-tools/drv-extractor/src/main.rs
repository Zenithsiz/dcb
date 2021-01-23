//! `.DRV` extractor

// Features
#![feature(array_value_iter)]

// Modules
mod cli;
mod logger;

// Imports
use anyhow::Context;
use dcb_io::{
	drv::{dir::entry::DirEntryKind, Dir},
	DrvFs,
};
use std::{
	io::{self, SeekFrom},
	path::Path,
};


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

	let drv_fs = DrvFs::from_reader(&mut input_file).context("Unable to parse filesystem")?;
	self::extract_tree(&mut input_file, drv_fs.root(), output_dir).context("Unable to extract files from root")
}

/// Extracts a `.drv` file from a reader and starting directory
fn extract_tree<R: io::Read + io::Seek>(drv_fs: &mut R, dir: &Dir, path: &Path) -> Result<(), anyhow::Error> {
	// Create path if it doesn't exist
	self::try_create_folder(path)?;

	// Then for each entry create it
	for entry in dir.entries() {
		match entry.kind {
			DirEntryKind::File { extension, size } => {
				let path = path.join(format!("{}.{}", entry.name, extension));

				log::info!("{} ({} bytes)", path.display(), size);

				// Seek the file and read it's size at most
				drv_fs
					.seek(SeekFrom::Start(u64::from(entry.sector_pos) * 2048))
					.with_context(|| format!("Unable to seek to directory {}", path.display()))?;
				let mut input_file = <&mut R as io::Read>::take(drv_fs, u64::from(size));

				// Then create the output file and copy.
				let mut output_file = std::fs::File::create(&path).with_context(|| format!("Unable to create file {}", path.display()))?;
				std::io::copy(&mut input_file, &mut output_file).with_context(|| format!("Unable to write file {}", path.display()))?;
			},
			DirEntryKind::Dir => {
				let path = path.join(entry.name.as_str());
				log::info!("{}", path.display());

				// Create the directory
				self::try_create_folder(&path)?;

				// Seek and read the directory on the input file
				drv_fs
					.seek(SeekFrom::Start(u64::from(entry.sector_pos) * 2048))
					.with_context(|| format!("Unable to seek to directory {}", path.display()))?;
				let dir = Dir::from_reader(drv_fs).with_context(|| format!("Unable to parse directory {}", path.display()))?;

				// Then recurse
				self::extract_tree(drv_fs, &dir, &path).with_context(|| format!("Unable to write directory {}", path.display()))?;
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
