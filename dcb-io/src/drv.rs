#![doc(include = "drv.md")]

// Modules
pub mod dir;
pub mod error;
pub mod file;

// Exports
pub use dir::{DirEntryReader, DirEntryWriter, DirReader, DirWriter};
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
	pub fn write_fs<W: io::Write + io::Seek, R: io::Read, I: ExactSizeIterator<Item = Result<DirEntryWriter<R, I>, io::Error>>>(
		writer: &mut W, root_entries: I,
	) -> Result<(), ToWriterError> {
		// Get the root and write it
		let root = DirWriter::new(root_entries);
		root.write_entries(writer).map_err(ToWriterError::RootDir)?;

		Ok(())
	}
}
