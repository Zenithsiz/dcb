//! CD-ROM/XA Implementation
//!
//! This module contains the implementation and abstraction
//! of the CD-ROM/XA Mode 2 Form 1 image file format used by
//! the ISO 9660 filesystem.

// Modules
pub mod error;
pub mod sector;

// Exports
pub use error::SectorError;
pub use sector::Sector;

// Imports
use std::io::{Read, Seek};

/// A CD-ROM/XA Mode 2 Form 1 wrapper
pub struct CdRomReader<R> {
	/// Underlying reader
	_reader: R,
}

// Constants
impl<R> CdRomReader<R> {
	/// Sector size
	pub const SECTOR_SIZE: u64 = 2352;
}

// Constructors
impl<R> CdRomReader<R> {
	/// Creates a new CD-ROM reader
	#[must_use]
	pub const fn new(reader: R) -> Self {
		Self { _reader: reader }
	}
}

// Read
impl<R: Read + Seek> CdRomReader<R> {
	/*
	/// Reads the `n`th sector
	pub fn sector(&mut self, n: u64) -> Result<Sector, SectorError> {
		// Seek to the sector.
		self.reader.seek(SeekFrom::Start(Self::SECTOR_SIZE * n)).map_err(SectorError::Seek)?;

		// Read it
		let mut bytes = [0; 2352];
		self.reader.read_exact(&mut bytes).map_err(SectorError::Read)?;

		// And parse it
		let sector = Sector::from_bytes(&bytes).map_err(SectorError::Parse)?;
		Ok(sector)
	}
	*/
}
