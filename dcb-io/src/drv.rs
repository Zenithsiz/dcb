#![doc(include = "drv.md")]

// Modules
pub mod dir;
pub mod error;
pub mod file;

// Exports
pub use dir::{DirReader, ReadDirEntry};
pub use error::FromReaderError;
pub use file::File;

// Imports
use std::io;

/// Filesystem reader
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct DrvFsReader {
	/// Root directory
	root: DirReader,
}

impl DrvFsReader {
	/// Reads a filesystem from a reader
	pub fn from_reader<R: io::Read>(reader: &mut R) -> Result<Self, FromReaderError> {
		// Read the root directory
		let root = DirReader::from_reader(reader).map_err(FromReaderError::RootDir)?;

		Ok(Self { root })
	}

	/// Returns the root directory of this filesystem
	#[must_use]
	pub const fn root(&self) -> &DirReader {
		&self.root
	}
}
