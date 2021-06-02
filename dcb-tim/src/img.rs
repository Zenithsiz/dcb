//! Image

// Modules
pub mod error;
pub mod header;

// Exports
pub use error::DeserializeError;
pub use header::Header;

// Imports
use crate::{BitsPerPixel, Color};
use byteorder::{LittleEndian, ReadBytesExt};
use dcb_bytes::Bytes;
use std::io;

/// Image
#[derive(PartialEq, Clone, Debug)]
pub struct Img {
	/// Header
	pub header: Header,

	/// Pixels
	pub pixels: Pixels,
}

impl Img {
	/// Deserializes an image
	#[bitmatch::bitmatch]
	pub fn deserialize<R: io::Read>(reader: &mut R, bpp: BitsPerPixel) -> Result<Self, DeserializeError> {
		// Read and parse the header
		let mut header_bytes = [0u8; 0xc];
		reader
			.read_exact(&mut header_bytes)
			.map_err(DeserializeError::ReadHeader)?;
		let header: Header = Header::deserialize_bytes(&header_bytes).into_ok();

		// If the width and height don't match the length, return Err
		// Note: The unscaled width should the one used to check the length, not the scaled
		let pixels_len = header.length - 0xc;
		let [width, height] = header.size;
		if pixels_len != usize::from(width) * usize::from(height) * 2 {
			return Err(DeserializeError::SizePixelsMismatch {
				width,
				height,
				bpp,
				pixels_len,
			});
		}

		// Then decode the pixels
		let pixels = Pixels::deserialize(reader, pixels_len, bpp).map_err(DeserializeError::ReadColors)?;

		Ok(Self { header, pixels })
	}

	/// Returns if this image is indexed
	#[must_use]
	pub const fn is_indexed(&self) -> bool {
		self.pixels.is_indexed()
	}
}

/// All pixels within an image
#[derive(PartialEq, Clone, Debug)]
pub enum Pixels {
	/// 4-bit indexed
	Index4Bit(Box<[usize]>),

	/// 8-bit indexed
	Index8Bit(Box<[usize]>),

	/// 16-bit color
	Color16Bit(Box<[Color]>),

	/// 24-bit color
	Color24Bit(Box<[Color]>),
}

impl Pixels {
	/// Deserializes all pixels within `reader`.
	#[bitmatch::bitmatch]
	pub(self) fn deserialize<R: io::Read>(
		reader: &mut R, pixels_len: usize, bpp: BitsPerPixel,
	) -> Result<Self, io::Error> {
		let colors = match bpp {
			BitsPerPixel::Index4Bit => Self::Index4Bit(
				<&mut R as io::Read>::bytes(reader)
					.take(pixels_len)
					.map(|value| {
						#[bitmatch]
						let "aaaa_bbbb" = value?;
						Ok([b, a].map(usize::from))
					})
					.collect::<Result<Vec<[usize; 2]>, io::Error>>()?
					.into_iter()
					.flatten()
					.collect(),
			),
			BitsPerPixel::Index8Bit => Self::Index8Bit(
				<&mut R as io::Read>::bytes(reader)
					.take(pixels_len)
					.map(|value| value.map(usize::from))
					.collect::<Result<_, _>>()?,
			),
			BitsPerPixel::Color16Bit => Self::Color16Bit(
				(0..pixels_len / 2)
					.map(|_| {
						let value = reader.read_u16::<LittleEndian>()?;
						Ok(Color::from_16bit(value))
					})
					.collect::<Result<_, io::Error>>()?,
			),
			BitsPerPixel::Color24Bit => Self::Color24Bit(
				(0..pixels_len / 3)
					.map(|_| {
						let byte1 = reader.read_u16::<LittleEndian>()?;
						let byte2 = reader.read_u16::<LittleEndian>()?;
						let byte3 = reader.read_u16::<LittleEndian>()?;

						Ok(Color::from_24bit([byte1, byte2, byte3]))
					})
					.collect::<Result<Vec<_>, io::Error>>()?
					.into_iter()
					.flatten()
					.collect(),
			),
		};

		Ok(colors)
	}

	/// Returns the bpp associated with these pixels
	#[must_use]
	pub const fn bpp(&self) -> BitsPerPixel {
		match self {
			Self::Index4Bit(_) => BitsPerPixel::Index4Bit,
			Self::Index8Bit(_) => BitsPerPixel::Index8Bit,
			Self::Color16Bit(_) => BitsPerPixel::Color16Bit,
			Self::Color24Bit(_) => BitsPerPixel::Color24Bit,
		}
	}

	/// Returns if these pixels are indexed
	#[must_use]
	pub const fn is_indexed(&self) -> bool {
		self.bpp().is_indexed()
	}
}
