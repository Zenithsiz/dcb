//! Directory lister

// Modules
pub mod error;

// Exports
pub use error::{NewError, NextError, ReadEntryError};

// Imports
use dcb_drv::{DirEntryWriter, DirEntryWriterKind, DirWriter, DirWriterLister};
use itertools::{Itertools, Position};
use std::{
	cmp::Ordering,
	convert::{TryFrom, TryInto},
	fs,
	io::Seek,
	path::{Path, PathBuf},
	time::SystemTime,
};

/// Directory list
#[derive(Debug)]
pub struct DirLister {
	/// All entries
	entries: Vec<DirEntry>,

	/// Depth
	depth: usize,
}

/// Directory entry
#[derive(Debug)]
pub struct DirEntry {
	/// Metadata
	metadata: fs::Metadata,

	/// Path
	path: PathBuf,
}

impl DirLister {
	/// Creates a new iterator from a path
	pub fn new(path: &Path, depth: usize) -> Result<Self, NewError> {
		// Read the directory entries
		let mut entries = fs::read_dir(path)
			.map_err(|err| NewError::ReadDir(path.to_path_buf(), err))?
			.map(|entry| match entry {
				Ok(entry) => Ok(DirEntry {
					metadata: entry.metadata().map_err(ReadEntryError::ReadMetadata)?,
					path:     entry.path(),
				}),
				Err(err) => Err(ReadEntryError::Read(err)),
			})
			.collect::<Result<Vec<_>, _>>()
			.map_err(|err| NewError::ReadEntries(path.to_path_buf(), err))?;

		// Then sort them by type and then name
		entries.sort_by(|lhs, rhs| {
			// Get if they're a directory
			let lhs_is_dir = lhs.metadata.file_type().is_dir();
			let rhs_is_dir = rhs.metadata.file_type().is_dir();

			// Sort directories first
			match (lhs_is_dir, rhs_is_dir) {
				(true, false) => return Ordering::Less,
				(false, true) => return Ordering::Greater,
				_ => (),
			}

			// Then compare by name
			lhs.path.file_name().cmp(&rhs.path.file_name())
		});

		Ok(Self { entries, depth })
	}
}

impl DirWriterLister for DirLister {
	type Error = NextError;
	type FileReader = fs::File;
}

impl IntoIterator for DirLister {
	type Item = Result<DirEntryWriter<Self>, <Self as DirWriterLister>::Error>;

	type IntoIter = impl Iterator<Item = Self::Item> + ExactSizeIterator;

	fn into_iter(self) -> Self::IntoIter {
		let depth = self.depth;
		self.entries.into_iter().with_position().map(move |entry| {
			let (entry, is_last) = {
				match entry {
					Position::First(entry) | Position::Middle(entry) => (entry, false),
					Position::Last(entry) | Position::Only(entry) => (entry, true),
				}
			};

			// Read the entry and it's metadata
			let name = entry
				.path
				.file_stem()
				.ok_or(NextError::NoEntryName)?
				.try_into()
				.map_err(NextError::InvalidEntryName)?;
			let secs_since_epoch = entry
				.metadata
				.modified()
				.map_err(NextError::EntryDate)?
				.duration_since(SystemTime::UNIX_EPOCH)
				.map_err(NextError::EntryDateSinceEpoch)?
				.as_secs();
			let date = chrono::NaiveDateTime::from_timestamp(
				i64::try_from(secs_since_epoch).map_err(|_err| NextError::EntryDateI64Secs)?,
				0,
			);

			// Check if it's a directory or file
			let kind = match entry.metadata.is_dir() {
				false => {
					let mut reader = fs::File::open(&entry.path).map_err(NextError::OpenFile)?;
					let extension = entry
						.path
						.extension()
						.ok_or(NextError::NoFileExtension)?
						.try_into()
						.map_err(NextError::InvalidFileExtension)?;
					let size = reader.stream_len().ok();

					let prefix = dcb_util::DisplayWrapper::new(|f| {
						match depth {
							0 => (),
							_ => {
								for _ in 0..(depth - 1) {
									write!(f, "│   ")?;
								}
								match is_last {
									true => write!(f, "└──")?,
									false => write!(f, "├──")?,
								};
							},
						}

						Ok(())
					});

					let size = dcb_util::DisplayWrapper::new(|f| match size {
						Some(size) => write!(f, "{}B", size_format::SizeFormatterSI::new(size)),
						None => write!(f, "Unknown Size"),
					});

					println!("{}{} ({})", prefix, name, size,);

					DirEntryWriterKind::File { extension, reader }
				},
				true => {
					let entries = Self::new(&entry.path, depth + 1).map_err(NextError::OpenDir)?;

					println!("{} ({} entries)", entry.path.display(), entries.entries.len());

					let dir = DirWriter::new(entries);
					DirEntryWriterKind::Dir(dir)
				},
			};

			Ok(DirEntryWriter { name, date, kind })
		})
	}
}
