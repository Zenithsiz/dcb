//! Directory lister

// Modules
pub mod error;

// Exports
pub use error::{DirListNewError, NextError};

// Imports
use dcb_io::drv::{dir::entry::DirEntryWriterKind, DirEntryWriter, DirWriter, DirWriterLister, FileWriter};
use std::{
	convert::{TryFrom, TryInto},
	fs,
	io::Seek,
	path::Path,
	time::SystemTime,
};

/// Directory list
#[derive(Debug)]
pub struct DirLister {
	/// Directory read
	dir: fs::ReadDir,

	/// Number of entries
	entries_len: u32,
}

impl DirLister {
	/// Creates a new iterator from a path
	pub fn new(path: &Path) -> Result<Self, DirListNewError> {
		// Get the length
		let len = fs::read_dir(path)
			.map_err(|err| DirListNewError::ReadDir(path.to_path_buf(), err))?
			.count();
		let entries_len = u32::try_from(len).map_err(|_err| DirListNewError::TooManyEntries)?;

		// And read the directory
		let dir = fs::read_dir(path).map_err(|err| DirListNewError::ReadDir(path.to_path_buf(), err))?;

		Ok(Self { dir, entries_len })
	}
}

impl DirWriterLister for DirLister {
	type DirList = Self;
	type Error = NextError;
	type FileReader = fs::File;

	fn entries_len(&self) -> u32 {
		self.entries_len
	}
}

impl Iterator for DirLister {
	type Item = Result<DirEntryWriter<<Self as DirWriterLister>::DirList>, <Self as DirWriterLister>::Error>;

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
					let mut file = fs::File::open(&path).map_err(NextError::OpenFile)?;
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
					let entries = Self::new(&path).map_err(NextError::OpenDir)?;

					log::info!("{} ({} entries)", path.display(), entries.entries_len);

					let dir = DirWriter::new(entries);
					DirEntryWriterKind::Dir(dir)
				},
			};

			DirEntryWriter::new(name, date, kind)
		};

		Some(res)
	}
}
