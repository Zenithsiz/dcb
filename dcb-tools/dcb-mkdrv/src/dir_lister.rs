//! Directory lister

// Modules
pub mod error;

// Exports
pub use error::{DirListNewError, NextError};

// Imports
use dcb_drv::{dir::entry::DirEntryWriterKind, DirEntryWriter, DirWriter, DirWriterLister, FileWriter};
use fs::FileType;
use std::{
	cmp::Ordering,
	convert::{TryFrom, TryInto},
	fs::{self, DirEntry},
	io::Seek,
	path::Path,
	time::SystemTime,
};

/// Directory list
#[derive(Debug)]
pub struct DirLister {
	/// All entries
	entries: Vec<DirEntry>,
}

impl DirLister {
	/// Creates a new iterator from a path
	pub fn new(path: &Path) -> Result<Self, DirListNewError> {
		// Read the directory entries
		let mut entries = fs::read_dir(path)
			.map_err(|err| DirListNewError::ReadDir(path.to_path_buf(), err))?
			.collect::<Result<Vec<_>, _>>()
			.map_err(|err| DirListNewError::ReadEntries(path.to_path_buf(), err))?;

		// If there are too many entries, return Err
		if u32::try_from(entries.len()).is_err() {
			return Err(DirListNewError::TooManyEntries);
		}

		// Then sort them by type and then name
		entries.sort_by(|lhs, rhs| {
			// Get if they're a directory
			// Note: If we can't read it, we just say it's a directory
			let lhs_is_dir = lhs.file_type().as_ref().map_or(false, FileType::is_dir);
			let rhs_is_dir = rhs.file_type().as_ref().map_or(false, FileType::is_dir);

			// Sort directories first
			match (lhs_is_dir, rhs_is_dir) {
				(true, false) => return Ordering::Less,
				(false, true) => return Ordering::Greater,
				_ => (),
			}

			// Then compare by name
			// TODO: Avoid allocations here?
			lhs.file_name().cmp(&rhs.file_name())
		});

		Ok(Self { entries })
	}
}

impl DirWriterLister for DirLister {
	type Error = NextError;
	type FileReader = fs::File;

	fn entries_len(&self) -> u32 {
		// Note: We makes sure it's less than `u32::MAX` in the constructor
		self.entries.len().try_into().expect("Too many entries")
	}
}

impl IntoIterator for DirLister {
	type Item = Result<DirEntryWriter<Self>, <Self as DirWriterLister>::Error>;

	type IntoIter = impl Iterator<Item = Self::Item>;

	fn into_iter(self) -> Self::IntoIter {
		self.entries.into_iter().map(|entry| {
			// Read the entry and it's metadata
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
			let kind = match metadata.is_dir() {
				false => {
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

					println!("{} ({} bytes)", path.display(), size);

					let file = FileWriter::new(extension, file, size);
					DirEntryWriterKind::File(file)
				},
				true => {
					let entries = Self::new(&path).map_err(NextError::OpenDir)?;

					println!("{} ({} entries)", path.display(), entries.entries.len());

					let dir = DirWriter::new(entries);
					DirEntryWriterKind::Dir(dir)
				},
			};

			Ok(DirEntryWriter::new(name, date, kind))
		})
	}
}
