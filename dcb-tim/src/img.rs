//! Image

// Modules
pub mod error;
pub mod header;

// Exports
pub use error::DeserializeError;
pub use header::Header;

// Imports
use crate::{header::BitsPerPixel, Color};
use byteorder::{LittleEndian, ReadBytesExt};
use dcb_bytes::Bytes;
use std::io;

/// Image
#[derive(PartialEq, Clone, Debug)]
pub struct Img {
	/// Header
	pub header: Header,

	/// Colors
	pub colors: Colors,
}

impl Img {
	/// Deserializes an image
	#[bitmatch::bitmatch]
	pub fn deserialize<R: io::Read>(reader: &mut R, bbp: BitsPerPixel) -> Result<Self, DeserializeError> {
		// Read and parse the header
		let mut header_bytes = [0u8; 0xc];
		reader
			.read_exact(&mut header_bytes)
			.map_err(DeserializeError::ReadHeader)?;
		let header: Header = Header::from_bytes(&header_bytes).into_ok();

		let colors = Colors::deserialize(reader, header.length - 0xc, bbp)?;

		Ok(Self { header, colors })
	}
}

/// Colors
#[derive(PartialEq, Clone, Debug)]
pub enum Colors {
	/// 4-bit indexed
	Index4Bit(Box<[usize]>),

	/// 8-bit indexed
	Index8Bit(Box<[usize]>),

	/// 16-bit color
	Color16Bit(Box<[Color]>),

	/// 24-bit color
	Color24Bit(Box<[Color]>),
}

impl Colors {
	/// Reads colors given a bbp
	#[bitmatch::bitmatch]
	pub(self) fn deserialize<R: io::Read>(
		reader: &mut R, length: usize, bbp: BitsPerPixel,
	) -> Result<Self, DeserializeError> {
		let colors = match bbp {
			BitsPerPixel::Index4Bit => Self::Index4Bit(
				<&mut R as io::Read>::bytes(reader)
					.take(length)
					.map(|value| {
						#[bitmatch]
						let "aaaa_bbbb" = value.map_err(DeserializeError::ReadColors)?;
						Ok([a, b].map(usize::from))
					})
					.collect::<Result<Vec<[usize; 2]>, _>>()?
					.into_iter()
					.flatten()
					.collect(),
			),
			BitsPerPixel::Index8Bit => Self::Index8Bit(
				<&mut R as io::Read>::bytes(reader)
					.take(length)
					.map(|value| value.map(usize::from).map_err(DeserializeError::ReadColors))
					.collect::<Result<_, _>>()?,
			),
			BitsPerPixel::Color16Bit => Self::Color16Bit(
				(0..length / 2)
					.map(|_| {
						let value = reader
							.read_u16::<LittleEndian>()
							.map_err(DeserializeError::ReadColors)?;
						Ok(Color::from_16bit(value))
					})
					.collect::<Result<_, _>>()?,
			),
			BitsPerPixel::Color24Bit => Self::Color24Bit(
				(0..length / 2)
					.map(|_| {
						let byte1 = reader
							.read_u16::<LittleEndian>()
							.map_err(DeserializeError::ReadColors)?;
						let byte2 = reader
							.read_u16::<LittleEndian>()
							.map_err(DeserializeError::ReadColors)?;
						let byte3 = reader
							.read_u16::<LittleEndian>()
							.map_err(DeserializeError::ReadColors)?;

						Ok(Color::from_24bit([byte1, byte2, byte3]))
					})
					.collect::<Result<Vec<_>, _>>()?
					.into_iter()
					.flatten()
					.collect(),
			),
		};

		Ok(colors)
	}
}
