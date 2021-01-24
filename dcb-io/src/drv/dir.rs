//! Directory

// Modules
pub mod entry;
pub mod error;

// Exports
pub use entry::DirEntry;
pub use error::{EntriesError, ReadEntryError};

// Imports
use dcb_bytes::Bytes;
use std::io::{self, SeekFrom};

/// Directory reader
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct DirReader {
	/// Sector position
	sector_pos: u32,
}

impl DirReader {
	/// Creates a directory reader from it's sector
	#[must_use]
	pub const fn new(sector_pos: u32) -> Self {
		Self { sector_pos }
	}

	/// Returns this directory's sector position
	#[must_use]
	pub const fn sector_pos(self) -> u32 {
		self.sector_pos
	}

	/// Seeks to this directory on a reader
	pub fn seek_to<R: io::Seek>(self, reader: &mut R) -> Result<u64, io::Error> {
		reader.seek(SeekFrom::Start(u64::from(self.sector_pos) * 2048))
	}

	/// Returns an iterator over all entries in this directory
	pub fn entries<R: io::Read + io::Seek>(
		self, reader: &mut R,
	) -> Result<impl Iterator<Item = Result<DirEntry, ReadEntryError>> + '_, EntriesError> {
		// Seek to the sector
		reader
			.seek(SeekFrom::Start(u64::from(self.sector_pos) * 2048))
			.map_err(EntriesError::Seek)?;

		// Then create the iterator
		let iter = std::iter::from_fn(move || {
			// Read the bytes
			let mut entry_bytes = [0; 0x20];
			if let Err(err) = reader.read_exact(&mut entry_bytes) {
				return Some(Err(ReadEntryError::ReadEntry(err)));
			}

			// And parse it
			match DirEntry::from_bytes(&entry_bytes) {
				Err(entry::FromBytesError::InvalidKind(0)) => None,
				res => Some(res.map_err(ReadEntryError::ParseEntry)),
			}
		});

		Ok(iter)
	}
}
