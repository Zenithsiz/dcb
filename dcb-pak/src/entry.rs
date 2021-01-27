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
use super::Header;
use dcb_bytes::Bytes;
use std::io;

/// A `.PAK` entry reader
#[derive(PartialEq, Eq, Debug)]
pub struct PakEntry<'a, R> {
	/// Header
	header: Header,

	/// Entry reader
	reader: &'a mut R,
}

impl<'a, R> PakEntry<'a, R> {
	/// Returns this entry's header
	#[must_use]
	pub const fn header(&self) -> &Header {
		&self.header
	}
}

impl<'a, R: io::Read> PakEntry<'a, R> {
	/// Deserializes an entry from a reader
	pub fn from_reader(reader: &'a mut R) -> Result<Option<Self>, FromReaderError> {
		// Read the header
		// Note: We do a two-part read so we can quit early if we find `0xffff`
		let mut header_bytes = [0u8; 0x8];
		reader.read_exact(&mut header_bytes[..0x4]).map_err(FromReaderError::ReadHeader)?;

		// If we found `0xFFFF`, this is the final header, return
		if header_bytes[..0x4] == [0xff, 0xff, 0xff, 0xff] {
			return Ok(None);
		}

		// Then read the rest and parse the header
		reader.read_exact(&mut header_bytes[0x4..]).map_err(FromReaderError::ReadHeader)?;
		let header = Header::from_bytes(&header_bytes).map_err(FromReaderError::ParseHeader)?;

		Ok(Some(Self { header, reader }))
	}

	/// Returns the contents of this entry
	#[must_use]
	pub fn contents(self) -> impl io::Read + 'a {
		// Note: We left the reader at the start of the read, so
		//       this will be correct.
		<&mut R as io::Read>::take(self.reader, u64::from(self.header.size))
	}
}
