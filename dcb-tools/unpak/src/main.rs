//! `.Pak` extractor

// Features
#![feature(osstring_ascii)]

// Modules
mod cli;

// Imports
use anyhow::Context;
use cli::CliData;
use dcb_pak::{header, PakFileReader};
use std::{
	fs, io,
	path::{Path, PathBuf},
};


fn main() -> Result<(), anyhow::Error> {
	// Initialize the logger
	simplelog::TermLogger::init(log::LevelFilter::Info, simplelog::Config::default(), simplelog::TerminalMode::Stderr)
		.expect("Unable to initialize logger");

	// Get all data from cli
	let cli_data = CliData::new();

	// For each input file, extract it
	for input_file_path in &cli_data.input_files {
		// If we don't have an output, try the input filename without extension if it's `.pak`, else use `.`
		let output_dir = match &cli_data.output_dir {
			Some(output) => output.to_path_buf(),
			None => match input_file_path.extension() {
				Some(extension) if extension.eq_ignore_ascii_case("pak") => input_file_path.with_extension(""),
				_ => PathBuf::from("."),
			},
		};

		// Then extract the tree
		if let Err(err) = self::extract_file(&input_file_path, &output_dir, &cli_data) {
			log::error!("Unable to extract files from {}: {:?}", input_file_path.display(), err);
		}
	}

	Ok(())
}

/// Extracts a `.pak` file to `output_dir`.
fn extract_file(input_file: &Path, output_dir: &Path, cli_data: &CliData) -> Result<(), anyhow::Error> {
	// Open the file and parse a `pak` filesystem from it.
	let input_file = fs::File::open(input_file).context("Unable to open input file")?;
	let mut input_file = io::BufReader::new(input_file);
	let mut pak_fs = PakFileReader::new(&mut input_file);

	// Try to create the output directory if we're not just listing
	if !cli_data.only_list {
		dcb_util::try_create_folder(output_dir).with_context(|| format!("Unable to create directory {}", output_dir.display()))?;
	}

	// Then read all entries
	while let Some(entry) = pak_fs.next_entry().context("Unable to read entry")? {
		// Get the filename
		let filename = entry.header().id;

		// Get the extension
		let extension = match entry.header().kind {
			header::Kind::Model3DSet => "M3D",
			header::Kind::Unknown1 => "UN1",
			header::Kind::GameScript => "MSD",
			header::Kind::Animation2D => "A2D",
			header::Kind::Unknown2 => "UN2",
			header::Kind::FileContents => "BIN",
			header::Kind::AudioSeq => "SEQ",
			header::Kind::AudioVh => "VH",
			header::Kind::AudioVb => "VB",
		};

		let path = output_dir.join(format!("{}.{}", filename, extension));
		if !cli_data.quiet {
			println!(
				"{} ({}B)",
				path.display(),
				size_format::SizeFormatterSI::new(u64::from(entry.header().size))
			);
		}

		// If we're only listing, stop here
		if cli_data.only_list {
			continue;
		}

		// Seek the file and read it's size at most
		let mut contents = entry.contents();

		// Then create the output file and copy.
		if cli_data.warn_on_override && path.exists() {
			log::warn!("Overriding file {}", path.display());
		}

		let mut output_file = fs::File::create(&path).with_context(|| format!("Unable to create file {}", path.display()))?;
		io::copy(&mut contents, &mut output_file).with_context(|| format!("Unable to write file {}", path.display()))?;
	}

	Ok(())
}
