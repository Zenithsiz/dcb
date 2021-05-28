//! Pointers

// Modules
pub mod error;

// Exports
pub use error::{
	FileCursorError, FindEntryError, ReadEntriesError, ReadEntryError, WriteEntriesError, WriteEntryError,
};

// Imports
use super::DirEntry;
use crate::DirEntryKind;
use ascii::AsciiStr;
use dcb_util::IoCursor;
use std::{
	convert::TryFrom,
	io::{self, SeekFrom},
};

/// File pointer
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct FilePtr {
	/// Sector position
	pub sector_pos: u32,

	/// Size
	pub size: u32,
}

impl FilePtr {
	/// Creates a new file pointer
	#[must_use]
	pub const fn new(sector_pos: u32, size: u32) -> Self {
		Self { sector_pos, size }
	}

	/// Seeks to this directory on a cursor
	pub fn seek_to<T: io::Seek>(self, cursor: &mut T) -> Result<u64, io::Error> {
		cursor.seek(SeekFrom::Start(u64::from(self.sector_pos) * 0x800))
	}

	/// Returns a cursor for this file
	pub fn cursor<T: io::Seek>(self, cursor: T) -> Result<IoCursor<T>, FileCursorError> {
		let pos = u64::from(self.sector_pos) * 0x800;
		IoCursor::new(cursor, pos, u64::from(self.size)).map_err(FileCursorError::Seek)
	}
}

impl PartialOrd for FilePtr {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for FilePtr {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		// Only compare sector position
		self.sector_pos.cmp(&other.sector_pos)
	}
}

/// Directory pointer
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct DirPtr {
	/// Sector position
	pub sector_pos: u32,
}

impl DirPtr {
	/// Creates a new directory pointer
	#[must_use]
	pub const fn new(sector_pos: u32) -> Self {
		Self { sector_pos }
	}

	/// Returns the root directory pointer
	#[must_use]
	pub const fn root() -> Self {
		Self { sector_pos: 0 }
	}

	/// Seeks to this directory on a cursor
	pub fn seek_to<T: io::Seek>(self, cursor: &mut T) -> Result<u64, io::Error> {
		cursor.seek(SeekFrom::Start(u64::from(self.sector_pos) * 0x800))
	}

	/// Returns an iterator over all entries in this directory
	pub fn read_entries<R: io::Read + io::Seek>(
		self, reader: &mut R,
	) -> Result<impl Iterator<Item = Result<DirEntry, ReadEntryError>> + '_, ReadEntriesError> {
		// Seek to the sector
		self.seek_to(reader).map_err(ReadEntriesError::Seek)?;

		// Then create the iterator
		let iter = std::iter::from_fn(move || {
			let entry: Result<_, _> = try {
				// Read the bytes
				let mut entry_bytes = [0; 0x20];
				reader.read_exact(&mut entry_bytes).map_err(ReadEntryError::ReadEntry)?;

				// And parse it
				DirEntry::from_bytes(&entry_bytes).map_err(ReadEntryError::ParseEntry)?
			};

			entry.transpose()
		});

		Ok(iter)
	}

	/// Finds an entry
	pub fn find_entry<R: io::Read + io::Seek>(
		self, reader: &mut R, entry_name: &AsciiStr,
	) -> Result<(usize, DirEntry), FindEntryError> {
		let (filename, extension) = entry_name
			.as_str()
			.split_once('.')
			.map_or((entry_name.as_str(), None), |(filename, extension)| {
				(filename, Some(extension))
			});

		self.read_entries(reader)
			.map_err(FindEntryError::SeekDir)?
			.enumerate()
			.find_map(|(idx, entry)| match entry {
				Ok(entry) => {
					let is_match = entry.name.as_str() == filename &&
						match entry.kind {
							DirEntryKind::Dir { .. } => extension.is_none(),
							DirEntryKind::File { extension: ext, .. } => extension == Some(ext.as_str()),
						};

					match is_match {
						true => Some(Ok((idx, entry))),
						false => None,
					}
				},
				Err(err) => Some(Err(err)),
			})
			.ok_or(FindEntryError::FindEntry)?
			.map_err(FindEntryError::ReadEntry)
	}

	/// Writes a list of entries to a writer
	pub fn write_entries<W: io::Seek + io::Write>(
		self, writer: &mut W, entries: impl IntoIterator<Item = DirEntry>,
	) -> Result<(), WriteEntriesError> {
		// Seek to the sector
		self.seek_to(writer).map_err(WriteEntriesError::Seek)?;

		// For each entry, write it
		for entry in entries {
			// Put the entry into bytes
			let mut entry_bytes = [0; 0x20];
			entry.to_bytes(&mut entry_bytes);

			// Then write it
			writer.write_all(&entry_bytes).map_err(WriteEntriesError::WriteEntry)?;
		}

		Ok(())
	}

	/// Writes a single entry to this directory
	pub fn write_entry<W: io::Seek + io::Write>(
		self, writer: &mut W, entry: &DirEntry, idx: usize,
	) -> Result<(), WriteEntryError> {
		// Seek to the sector and then to the entry
		// TODO: Maybe do this in one seek
		self.seek_to(writer).map_err(WriteEntryError::Seek)?;
		writer
			.seek(SeekFrom::Current(
				0x20 * i64::try_from(idx).expect("Entry index didn't fit into `usize`"),
			))
			.map_err(WriteEntryError::Seek)?;

		// Then write the entry
		let mut entry_bytes = [0; 0x20];
		entry.to_bytes(&mut entry_bytes);

		// Then write it
		writer.write_all(&entry_bytes).map_err(WriteEntryError::WriteEntry)
	}
}
