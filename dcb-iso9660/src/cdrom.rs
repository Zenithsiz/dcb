#![doc(include = "cdrom.md")]

// Modules
pub mod error;
pub mod iter;
pub mod sector;

// Exports
pub use error::ReadSectorError;
pub use iter::SectorsRangeIter;
pub use sector::Sector;

// Imports
use dcb_bytes::Bytes;
use std::{
	io::{Read, Seek, SeekFrom},
	ops::RangeBounds,
};

/// A CD-ROM/XA Mode 2 Form 1 wrapper
pub struct CdRom<R> {
	/// Underlying reader
	reader: R,
}

// Constants
impl<R> CdRom<R> {
	/// Sector size
	pub const SECTOR_SIZE: u64 = 2352;
}

// Constructors
impl<R> CdRom<R> {
	/// Creates a new CD-ROM reader
	#[must_use]
	pub const fn new(reader: R) -> Self {
		Self { reader }
	}
}

// Read
impl<R: Read + Seek> CdRom<R> {
	/// Reads the `n`th sector
	pub fn read_sector(&mut self, n: u64) -> Result<Sector, ReadSectorError> {
		// Seek to the sector.
		self.reader.seek(SeekFrom::Start(Self::SECTOR_SIZE * n)).map_err(ReadSectorError::Seek)?;

		// Read it
		let mut bytes = [0; 2352];
		self.reader.read_exact(&mut bytes).map_err(ReadSectorError::Read)?;

		// And parse it
		let sector = Sector::from_bytes(&bytes).map_err(ReadSectorError::Parse)?;
		Ok(sector)
	}

	/// Returns an iterator over a range of sectors
	pub fn read_sectors_range(&mut self, range: impl RangeBounds<u64>) -> SectorsRangeIter<R> {
		SectorsRangeIter::new(self, range)
	}
}
