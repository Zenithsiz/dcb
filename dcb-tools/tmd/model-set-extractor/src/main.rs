//! Model set extractor

// Modules
mod cli;
mod logger;

// Imports
use anyhow::Context;
use dcb_pak::reader::entry::Model3dSet;
use std::{
	io::{Read, Seek, SeekFrom},
	path::Path,
};


fn main() -> Result<(), anyhow::Error> {
	// Initialize the logger
	logger::init();

	// Get all data from cli
	let cli::CliData { input_file, output_dir } = cli::CliData::new();

	// Try to extract the file
	self::extract_file(&input_file, &output_dir).context("Unable to extract model set file")?;

	Ok(())
}

/// Extracts a model set to `output_dir`.
fn extract_file(input_file: &Path, output_dir: &Path) -> Result<(), anyhow::Error> {
	// Open the file and parse the model set from it.
	let mut input_file = std::fs::File::open(input_file).context("Unable to open input file")?;

	let model_set = Model3dSet::from_reader(&mut input_file).context("Unable to parse file")?;

	zutil::try_create_dir_all(output_dir)
		.with_context(|| format!("Unable to create directory {}", output_dir.display()))?;
	for (idx, (pos, size, ..)) in model_set.models.iter().enumerate() {
		// Get the filename
		let path = output_dir.join(format!("{}.TMD", idx));

		log::info!("{} ({} bytes)", path.display(), size);

		// Seek the file and read it's size at most
		input_file
			.seek(SeekFrom::Start(*pos))
			.with_context(|| format!("Unable to seek to file {}", path.display()))?;
		let mut input_file = input_file.by_ref().take(*size);

		// Then create the output file and copy.
		if path.exists() {
			log::warn!("Overriding file {}", path.display());
		}
		let mut output_file =
			std::fs::File::create(&path).with_context(|| format!("Unable to create file {}", path.display()))?;
		std::io::copy(&mut input_file, &mut output_file)
			.with_context(|| format!("Unable to write file {}", path.display()))?;
	}

	Ok(())
}
