//! A `.PAK` entry

// Modules
pub mod animation2d;
pub mod error;
pub mod model3d_set;

// Exports
pub use animation2d::Animation2d;
pub use error::FromReaderError;
pub use model3d_set::Model3dSet;

// Imports
use crate::Header;
use dcb_bytes::Bytes;
use std::io;

/// A `.PAK` entry reader
#[derive(PartialEq, Eq, Debug)]
pub struct PakEntryReader<'a, R> {
	/// Header
	header: Header,

	/// Entry reader
	///
	/// Note: This will be seeked to the start of
	///       this entry's content.
	reader: &'a mut R,
}

// Constructor
impl<'a, R: io::Read> PakEntryReader<'a, R> {
	/// Deserializes an entry from a reader
	pub fn from_reader(reader: &'a mut R) -> Result<Option<Self>, FromReaderError> {
		// Read the header
		// Note: We do a two-part read so we can quit early if we find `0xffffffff`
		let mut header_bytes = [0u8; 0x8];
		reader
			.read_exact(&mut header_bytes[..0x4])
			.map_err(FromReaderError::ReadHeader)?;

		// If we found `0xffffffff`, this is the final header, return
		if header_bytes[..0x4] == [0xff; 4] {
			return Ok(None);
		}

		// Then read the rest and parse the header
		reader
			.read_exact(&mut header_bytes[0x4..])
			.map_err(FromReaderError::ReadHeader)?;
		let header = Header::from_bytes(&header_bytes).map_err(FromReaderError::ParseHeader)?;

		Ok(Some(Self { header, reader }))
	}
}

// Getters
impl<'a, R> PakEntryReader<'a, R> {
	/// Returns this entry's header
	#[must_use]
	pub const fn header(&self) -> &Header {
		&self.header
	}
}

// Read
impl<'a, R: io::Read> PakEntryReader<'a, R> {
	/// Returns the contents of this entry
	#[must_use]
	pub fn contents(self) -> impl io::Read + 'a {
		// Note: Our reader is at the start of the contents
		<&mut R as io::Read>::take(self.reader, u64::from(self.header.size))
	}
}
