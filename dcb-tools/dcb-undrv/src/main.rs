//! `.DRV` extractor

// Modules
mod cli;

// Imports
use anyhow::Context;
use cli::CliData;
use dcb_drv::{DirEntryKind, DirPtr};
use std::{
	fs, io,
	path::{Path, PathBuf},
};

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

	// For each input file, extract it
	for input_file_path in &cli_data.input_files {
		// If we don't have an output, try the input filename without extension if it's `.drv`, else use `.`
		let output_dir = match &cli_data.output_dir {
			Some(output) => output.to_path_buf(),
			None => match input_file_path.extension() {
				Some(extension) if extension.eq_ignore_ascii_case("drv") => input_file_path.with_extension(""),
				_ => PathBuf::from("."),
			},
		};

		// Open the file and parse a `drv` filesystem from it.
		let input_file = fs::File::open(&input_file_path).context("Unable to open input file")?;
		let input_file_metadata = input_file.metadata().context("Unable to get input file metadata")?;
		let mut input_file = io::BufReader::new(input_file);

		// Create output directory if it doesn't exist
		dcb_util::try_create_folder(&output_dir)
			.with_context(|| format!("Unable to create directory {}", output_dir.display()))?;

		// Then extract the tree
		if let Err(err) = self::extract_tree(&mut input_file, DirPtr::root(), &output_dir, &cli_data) {
			log::error!("Unable to extract files from {}: {:?}", input_file_path.display(), err);
		}

		// And set it's time to the input file
		let time = filetime::FileTime::from_last_modification_time(&input_file_metadata);
		if let Err(err) = filetime::set_file_mtime(&output_dir, time) {
			log::warn!(
				"Unable to write date for output directory {}: {}",
				output_dir.display(),
				dcb_util::fmt_err_wrapper(&err)
			);
		}
	}

	Ok(())
}

/// Extracts a `.drv` file from a reader and starting directory
fn extract_tree<R: io::Read + io::Seek>(
	reader: &mut R, dir_ptr: DirPtr, path: &Path, cli_data: &CliData,
) -> Result<(), anyhow::Error> {
	// Get all entries
	// Note: We need to collect to free the reader so it can seek to the next files.
	let entries: Vec<_> = dir_ptr
		.entries(reader)
		.with_context(|| format!("Unable to get directory entries of {}", path.display()))?
		.collect();

	// Then extract each entry
	for entry in entries {
		// If we can't read it, return Err
		let entry = entry.with_context(|| format!("Unable to read directory entry of {}", path.display()))?;

		// Create the date
		// Note: `.DRV` only supports second precision.
		let time = filetime::FileTime::from_unix_time(entry.date.timestamp(), 0);

		// Then check it's type
		match entry.kind {
			// If it's a file, create the file and write all contents
			DirEntryKind::File { extension, ptr } => {
				let path = path.join(format!("{}.{}", entry.name, extension));

				// Log the file and it's size
				if !cli_data.quiet {
					println!(
						"{} ({}B)",
						path.display(),
						size_format::SizeFormatterSI::new(u64::from(ptr.size))
					);
				}

				// If the output file already exists, log a warning
				if cli_data.warn_on_override && path.exists() {
					log::warn!("Overriding file {}", path.display());
				}

				// Get the file's reader.
				let mut file_reader = ptr
					.cursor(&mut *reader)
					.with_context(|| format!("Unable to read file {}", path.display()))?;

				// Then create the output file and copy.
				let mut output_file =
					fs::File::create(&path).with_context(|| format!("Unable to create file {}", path.display()))?;
				std::io::copy(&mut file_reader, &mut output_file)
					.with_context(|| format!("Unable to write file {}", path.display()))?;

				// And set the file's modification time
				if let Err(err) = filetime::set_file_handle_times(&output_file, None, Some(time)) {
					log::warn!(
						"Unable to write date for file {}: {}",
						path.display(),
						dcb_util::fmt_err_wrapper(&err)
					);
				}
			},

			// If it's a directory, create it and recurse for all it's entries
			DirEntryKind::Dir { ptr } => {
				let path = path.join(entry.name.as_str());

				// Log the directory
				if !cli_data.quiet {
					println!("{}/", path.display());
				}

				// Create the directory and recurse over it
				dcb_util::try_create_folder(&path)
					.with_context(|| format!("Unable to create directory {}", path.display()))?;
				self::extract_tree(reader, ptr, &path, &cli_data)
					.with_context(|| format!("Unable to extract directory {}", path.display()))?;

				// Then set it's date
				// Note: We must do this *after* extracting the tree, else the time
				//       will be updated when we insert files into it.
				if let Err(err) = filetime::set_file_mtime(&path, time) {
					log::warn!(
						"Unable to write date for directory {}: {}",
						path.display(),
						dcb_util::fmt_err_wrapper(&err)
					);
				}
			},
		}
	}

	Ok(())
}
