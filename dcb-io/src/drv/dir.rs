//! Directories

// Modules
pub mod entry;
pub mod error;

// Exports
pub use entry::DirEntry;
pub use error::DeserializeError;

// Imports
use dcb_bytes::Bytes;
use std::io;

/// Directory
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Dir {
	/// All directory entries
	entries: Vec<DirEntry>,
}

impl Dir {
	/// Parses a directory from a reader
	pub fn from_reader<R: io::Read>(mut reader: R) -> Result<Self, DeserializeError> {
		let entries = std::iter::from_fn(move || {
			// Read the bytes
			let mut entry_bytes = [0; 0x20];
			if let Err(err) = reader.read_exact(&mut entry_bytes) {
				return Some(Err(DeserializeError::ReadEntry(err)));
			}

			// And parse it
			match DirEntry::from_bytes(&entry_bytes) {
				Err(entry::FromBytesError::InvalidKind(0)) => None,
				res => Some(res.map_err(DeserializeError::ParseEntry)),
			}
		})
		.collect::<Result<Vec<_>, _>>()?;

		Ok(Self { entries })
	}

	/// Returns all the entries in this directory
	#[must_use]
	pub fn entries(&self) -> &[DirEntry] {
		&self.entries
	}
}
