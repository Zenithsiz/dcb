//! File pointers

// Modules
mod error;

// Exports
pub use error::FileCursorError;

// Imports
use std::io::{self, SeekFrom};
use zutil::IoSlice;

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
	pub fn cursor<T: io::Seek>(self, cursor: T) -> Result<IoSlice<T>, FileCursorError> {
		let pos = u64::from(self.sector_pos) * 0x800;
		IoSlice::new_with_offset_len(cursor, pos, u64::from(self.size)).map_err(FileCursorError::Seek)
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
