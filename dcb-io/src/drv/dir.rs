//! Directories

// Modules
pub mod entry;
pub mod error;

// Exports
pub use entry::ReadDirEntry;
pub use error::FromReaderError;

// Imports
use dcb_bytes::Bytes;
use std::io;

/// Directory reader
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct DirReader {
	/// All directory entries
	entries: Vec<ReadDirEntry>,
}

impl DirReader {
	/// Parses a directory from a reader
	pub fn from_reader<R: io::Read>(reader: &mut R) -> Result<Self, FromReaderError> {
		let entries = std::iter::from_fn(move || {
			// Read the bytes
			let mut entry_bytes = [0; 0x20];
			if let Err(err) = reader.read_exact(&mut entry_bytes) {
				return Some(Err(FromReaderError::ReadEntry(err)));
			}

			// And parse it
			match ReadDirEntry::from_bytes(&entry_bytes) {
				Err(entry::FromBytesError::InvalidKind(0)) => None,
				res => Some(res.map_err(FromReaderError::ParseEntry)),
			}
		})
		.collect::<Result<Vec<_>, _>>()?;

		Ok(Self { entries })
	}

	/// Returns all the entries in this directory
	#[must_use]
	pub fn entries(&self) -> &[ReadDirEntry] {
		&self.entries
	}
}
