//! Abstraction over the game file.
//!
//! See [`GameFile`] for details

// Modules
pub mod error;
pub mod sector;

// Exports
pub use error::SectorError;
pub use sector::Sector;

// Imports
use dcb_bytes::Bytes;
use std::io::{Read, Seek, SeekFrom};

/// Game file reader.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default, Hash, Debug)]
pub struct GameFile<R> {
	/// The type to read and write from
	reader: R,
}

impl<R> GameFile<R> {
	/// Sector size
	pub const SECTOR_SIZE: u64 = 2352;
}

// Constructors
impl<R> GameFile<R> {
	/// Creates a new game file from any reader
	pub const fn new(reader: R) -> Self {
		Self { reader }
	}
}

// Read
impl<R: Read + Seek> GameFile<R> {
	/// Reads the `n`th sector
	pub fn sector(&mut self, n: u64) -> Result<Sector, SectorError> {
		// Seek to the sector.
		self.reader.seek(SeekFrom::Start(Self::SECTOR_SIZE * n)).map_err(SectorError::Seek)?;

		// Read it
		let mut bytes = [0u8; <<Sector as Bytes>::ByteArray as dcb_bytes::ByteArray>::SIZE];
		self.reader.read_exact(&mut bytes).map_err(SectorError::Read)?;

		// And parse it
		let sector = Sector::from_bytes(&bytes).map_err(SectorError::Parse)?;
		Ok(sector)
	}
}
