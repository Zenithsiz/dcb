#![doc(include = "drv.md")]

// Modules
pub mod dir;
pub mod error;
pub mod file;

// Exports
pub use dir::{DirEntry, DirReader};
pub use error::FromReaderError;
pub use file::FileReader;

/// Filesystem reader
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct DrvFsReader;

impl DrvFsReader {
	/// Returns the root directory of this filesystem
	#[must_use]
	pub const fn root() -> DirReader {
		DirReader::new(0)
	}
}
