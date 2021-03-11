//! `.Pak` extractor

// Modules
mod cli;

// Imports
use anyhow::Context;
use dcb_pak::{header, PakFileReader};
use std::path::Path;


fn main() -> Result<(), anyhow::Error> {
	// Initialize the logger
	simplelog::TermLogger::init(log::LevelFilter::Info, simplelog::Config::default(), simplelog::TerminalMode::Stderr)
		.expect("Unable to initialize logger");

	// Get all data from cli
	let cli::CliData { input_file, output_dir } = cli::CliData::new();

	// Try to extract the filesystem
	self::extract_file(&input_file, &output_dir).context("Unable to extract `pak` file")?;

	Ok(())
}

/// Extracts a `.pak` file to `output_dir`.
fn extract_file(input_file: &Path, output_dir: &Path) -> Result<(), anyhow::Error> {
	// Open the file and parse a `pak` filesystem from it.
	let input_file = std::fs::File::open(input_file).context("Unable to open input file")?;
	let mut input_file = std::io::BufReader::new(input_file);
	let mut pak_fs = PakFileReader::new(&mut input_file);

	// Try to create the output directory
	self::try_create_folder(output_dir)?;

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

		log::info!("{} ({} bytes)", path.display(), entry.header().size);

		// Seek the file and read it's size at most
		let mut contents = entry.contents();

		// Then create the output file and copy.
		if path.exists() {
			log::warn!("Overriding file {}", path.display());
		}
		let mut output_file = std::fs::File::create(&path).with_context(|| format!("Unable to create file {}", path.display()))?;
		std::io::copy(&mut contents, &mut output_file).with_context(|| format!("Unable to write file {}", path.display()))?;
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
