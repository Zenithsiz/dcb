#![doc(include = "drv.md")]

// Modules
pub mod dir;
pub mod error;
pub mod file;

// Exports
pub use dir::{DirEntryReader, DirEntryWriter, DirReader, DirWriter, DirWriterLister};
pub use error::{FromReaderError, ToWriterError};
pub use file::{FileReader, FileWriter};

// Imports
use std::io;

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

/// Filesystem writer
pub struct DrvFsWriter;

impl DrvFsWriter {
	/// Creates a `.DRV` filesystem
	pub fn write_fs<W: io::Write + io::Seek, L: DirWriterLister>(
		writer: &mut W, root_entries: L, root_entries_len: u32,
	) -> Result<(), ToWriterError<L::Error>> {
		// Get the root and write it
		let root = DirWriter::new(root_entries, root_entries_len);
		root.write_entries(writer).map_err(ToWriterError::RootDir)?;

		Ok(())
	}
}
