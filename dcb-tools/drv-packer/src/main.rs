//! `.DRV` packer

// Features
#![feature(array_value_iter, try_blocks, seek_convenience)]

// Modules
mod cli;
mod logger;

// Imports
use anyhow::Context;
use dcb_io::drv::{dir::entry::DirEntryWriterKind, DirEntryWriter, DirWriter, DirWriterList, DrvFsWriter, FileWriter};
use std::{
	convert::{TryFrom, TryInto},
	fs,
	io::{self, Seek},
	path::{Path, PathBuf},
	time::SystemTime,
};


fn main() -> Result<(), anyhow::Error> {
	// Initialize the logger
	logger::init();

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
	let (root_entries, root_entries_len) = DirList::new(input_dir).context("Unable to read root directory")?;
	DrvFsWriter::write_fs(&mut output_file, root_entries, root_entries_len).context("Unable to write filesystem")
}

/// Directory list
#[derive(Debug)]
struct DirList {
	/// Directory read
	dir: fs::ReadDir,
}

impl DirList {
	/// Creates a new iterator from a path
	fn new(path: &Path) -> Result<(Self, u32), DirListNewError> {
		// Get the length
		let len = fs::read_dir(path)
			.map_err(|err| DirListNewError::ReadDir(path.to_path_buf(), err))?
			.count();
		let len = u32::try_from(len).map_err(|_err| DirListNewError::TooManyEntries)?;

		// And read the directory
		let dir = fs::read_dir(path).map_err(|err| DirListNewError::ReadDir(path.to_path_buf(), err))?;

		Ok((Self { dir }, len))
	}
}

/// Error for [`DirList::new`]
#[derive(Debug, thiserror::Error)]
enum DirListNewError {
	/// Unable to read directory
	#[error("Unable to read directory {}", _0.display())]
	ReadDir(PathBuf, #[source] io::Error),

	/// Too many entries in directory
	#[error("Too many entries in directory")]
	TooManyEntries,
}

/// Error for [`Iterator::Item`]
#[derive(Debug, thiserror::Error)]
enum NextError {
	/// Unable to read entry
	#[error("Unable to read entry")]
	ReadEntry(#[source] io::Error),

	/// Unable to read entry metadata
	#[error("Unable to read entry metadata")]
	ReadMetadata(#[source] io::Error),

	/// Entry had no name
	#[error("Entry had no name")]
	NoEntryName,

	/// Invalid file name
	#[error("Invalid file name")]
	InvalidEntryName(#[source] dcb_util::ascii_str_arr::FromBytesError<0x10>),

	/// File had no file name
	#[error("file had no file name")]
	NoFileExtension,

	/// Invalid extension
	#[error("Invalid extension")]
	InvalidFileExtension(#[source] dcb_util::ascii_str_arr::FromBytesError<0x3>),

	/// Unable to get entry date
	#[error("Unable to get entry date")]
	EntryDate(#[source] io::Error),

	/// Unable to get entry date as time since epoch
	#[error("Unable to get entry date as time since epoch")]
	EntryDateSinceEpoch(#[source] std::time::SystemTimeError),

	/// Unable to get entry date as `i64` seconds since epoch
	#[error("Unable to get entry date as `i64` seconds since epoch")]
	EntryDateI64Secs,

	/// Unable to open file
	#[error("Unable to open file")]
	OpenFile(#[source] io::Error),

	/// Unable to get file size
	#[error("Unable to get file size")]
	FileSize(#[source] io::Error),

	/// File was too big
	#[error("File was too big")]
	FileTooBig,

	/// Unable to open directory
	#[error("Unable to open directory")]
	OpenDir(#[source] DirListNewError),
}

impl DirWriterList for DirList {
	type DirList = Self;
	type Error = NextError;
	type FileReader = std::fs::File;
	type Iter = Self;

	fn into_iter(self) -> Self::Iter {
		self
	}
}

impl Iterator for DirList {
	type Item = Result<DirEntryWriter<<Self as DirWriterList>::DirList>, <Self as DirWriterList>::Error>;

	fn next(&mut self) -> Option<Self::Item> {
		// Get the next entry
		let entry = self.dir.next()?;

		// Then read it
		let res = try {
			// Read the entry and it's metadata
			let entry = entry.map_err(NextError::ReadEntry)?;
			let metadata = entry.metadata().map_err(NextError::ReadMetadata)?;
			let path = entry.path();
			let name = path
				.file_stem()
				.ok_or(NextError::NoEntryName)?
				.try_into()
				.map_err(NextError::InvalidEntryName)?;
			let secs_since_epoch = metadata
				.modified()
				.map_err(NextError::EntryDate)?
				.duration_since(SystemTime::UNIX_EPOCH)
				.map_err(NextError::EntryDateSinceEpoch)?
				.as_secs();
			let date = chrono::NaiveDateTime::from_timestamp(i64::try_from(secs_since_epoch).map_err(|_err| NextError::EntryDateI64Secs)?, 0);

			// Check if it's a directory or file
			let kind = match metadata.is_file() {
				true => {
					let mut file = std::fs::File::open(&path).map_err(NextError::OpenFile)?;
					let size = file
						.stream_len()
						.map_err(NextError::FileSize)?
						.try_into()
						.map_err(|_err| NextError::FileTooBig)?;
					let extension = path
						.extension()
						.ok_or(NextError::NoFileExtension)?
						.try_into()
						.map_err(NextError::InvalidFileExtension)?;

					log::info!("{} ({} bytes)", path.display(), size);

					let file = FileWriter::new(extension, file, size);
					DirEntryWriterKind::File(file)
				},
				false => {
					let (entries, entries_len) = Self::new(&path).map_err(NextError::OpenDir)?;

					log::info!("{} ({} entries)", path.display(), entries_len);

					let dir = DirWriter::new(entries, entries_len);
					DirEntryWriterKind::Dir(dir)
				},
			};

			DirEntryWriter::new(name, date, kind)
		};

		Some(res)
	}
}
