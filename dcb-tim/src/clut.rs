//! Clut

// Modules
pub mod error;
pub mod header;

// Exports
pub use error::DeserializeError;
pub use header::Header;

// Imports
use crate::Color;
use byteorder::{LittleEndian, ReadBytesExt};
use dcb_bytes::Bytes;
use std::io;

/// Color lookup table
#[derive(PartialEq, Clone, Debug)]
pub struct Clut {
	/// Header
	pub header: Header,

	/// Colors
	pub colors: Box<[Color]>,
}

impl Clut {
	/// Deserializes a clut table
	pub fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, DeserializeError> {
		// Read and parse the header
		let mut header_bytes = [0u8; 0xc];
		reader
			.read_exact(&mut header_bytes)
			.map_err(DeserializeError::ReadHeader)?;
		let header = Header::deserialize_bytes(&header_bytes).into_ok();

		let colors = (0..header.colors_len())
			.map(|_| {
				let value = reader
					.read_u16::<LittleEndian>()
					.map_err(DeserializeError::ReadColors)?;
				Ok(Color::from_16bit(value))
			})
			.collect::<Result<_, _>>()?;

		Ok(Self { header, colors })
	}
}
