//! Pointers

// Modules
pub mod error;

// Exports
pub use error::{FileCursorError, ReadEntriesError, ReadEntryError, WriteEntriesError};

// Imports
use super::DirEntry;
use dcb_util::IoCursor;
use std::io::{self, SeekFrom};

/// File pointer
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct FilePtr {
	/// Sector position
	pub sector_pos: u32,

	/// Size
	pub size: u32,
}

impl FilePtr {
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

/// Directory pointer
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct DirPtr {
	/// Sector position
	pub sector_pos: u32,
}

impl DirPtr {
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
}
