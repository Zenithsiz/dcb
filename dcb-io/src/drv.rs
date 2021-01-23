#![doc(include = "drv.md")]

// Modules
pub mod dir;
pub mod error;
pub mod file;

// Exports
pub use dir::{Dir, DirEntry};
pub use error::FromBytesError;
pub use file::File;

// Imports
use std::io;

/// The filesystem
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct DrvFs {
	/// Root directory
	root: Dir,
}

impl DrvFs {
	/// Reads a filesystem from a reader
	pub fn from_reader<R: io::Read>(reader: R) -> Result<Self, FromBytesError> {
		// Read the root directory
		let root = Dir::from_reader(reader).map_err(FromBytesError::RootDir)?;

		Ok(Self { root })
	}

	/// Returns the root directory of this filesystem
	#[must_use]
	pub const fn root(&self) -> &Dir {
		&self.root
	}
}
