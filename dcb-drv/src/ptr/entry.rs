//! Directory entry pointer

// Modules
mod error;

// Exports
pub use error::WriteEntryError;

// Imports
use crate::{DirEntry, DirPtr};
use dcb_bytes::Bytes;
use std::io::{self, SeekFrom};

/// Directory entry pointer
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct DirEntryPtr {
	/// Directory
	dir: DirPtr,

	/// Entry
	entry: u32,
}

impl DirEntryPtr {
	/// Creates a new entry pointer
	#[must_use]
	pub const fn new(dir: DirPtr, entry: u32) -> Self {
		Self { dir, entry }
	}

	/// Seeks to this entry on a cursor
	pub fn seek_to<T: io::Seek>(self, cursor: &mut T) -> Result<u64, io::Error> {
		cursor.seek(SeekFrom::Start(
			u64::from(self.dir.sector_pos) * 0x800 + u64::from(self.entry) * 0x20,
		))
	}

	/// Writes an entry to this pointer
	pub fn write<W: io::Seek + io::Write>(self, writer: &mut W, entry: &DirEntry) -> Result<(), WriteEntryError> {
		// Seek to this entry
		self.seek_to(writer).map_err(WriteEntryError::Seek)?;

		// Then write the entry
		let mut entry_bytes = [0; 0x20];
		entry.serialize_bytes(&mut entry_bytes).into_ok();

		// Then write it
		writer.write_all(&entry_bytes).map_err(WriteEntryError::WriteEntry)
	}
}
