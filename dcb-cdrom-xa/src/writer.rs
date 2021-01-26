//! Reader

// Modules
pub mod error;

// Exports
pub use error::WriteSectorError;

// Imports
use crate::Sector;
use dcb_bytes::Bytes;
use std::io::Write;

/// A CD-ROM/XA Mode 2 Form 1 writer.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct CdRomWriter<W> {
	/// Underlying writer
	writer: W,
}

// Constants
impl<W> CdRomWriter<W> {
	/// Sector size
	pub const SECTOR_SIZE: u64 = 2352;
}

// Constructors
impl<W> CdRomWriter<W> {
	/// Creates a new CD-ROM reader
	#[must_use]
	pub const fn new(writer: W) -> Self {
		Self { writer }
	}
}

// Write
impl<W: Write> CdRomWriter<W> {
	/// Writes the next sector
	pub fn write_sector(&mut self, sector: &Sector) -> Result<(), WriteSectorError> {
		// Serialize it
		let mut bytes = [0; 2352];
		sector.to_bytes(&mut bytes).map_err(WriteSectorError::ToBytes)?;

		// And write it
		self.writer.write_all(&bytes).map_err(WriteSectorError::Write)
	}
}
