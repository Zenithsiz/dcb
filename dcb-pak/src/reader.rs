//! `.PAK` file reader

// Modules
pub mod error;

// Exports
pub use error::FromReaderError;

// Imports
use super::PakEntry;
use std::io::{self, SeekFrom};

/// A `.PAK` file reader
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct PakFileReader<R> {
	/// Reader
	reader: R,

	/// Current position
	cur_pos: u64,
}

impl<R> PakFileReader<R> {
	/// Creates a new reader
	pub const fn new(reader: R) -> Self {
		Self { reader, cur_pos: 0 }
	}
}


// Constructor
impl<R: io::Read + io::Seek> PakFileReader<R> {
	/// Returns the next entry
	pub fn next_entry(&mut self) -> Result<Option<PakEntry<R>>, FromReaderError> {
		// Seek to our current position
		self.reader.seek(SeekFrom::Start(self.cur_pos)).map_err(FromReaderError::SeekNextEntry)?;

		// Try to read an entry
		let entry = match PakEntry::from_reader(&mut self.reader).map_err(FromReaderError::ReadEntry)? {
			Some(entry) => {
				// Note: `0x8` is the size of the header.
				self.cur_pos += 0x8 + u64::from(entry.header().size);
				Some(entry)
			},
			None => None,
		};

		Ok(entry)
	}
}
