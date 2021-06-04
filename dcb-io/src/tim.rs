//! `.TIM` parser

// Modules
pub mod clut;
pub mod error;

// Exports
pub use clut::Clut;
pub use error::DeserializeError;

// Imports
use byteorder::{ByteOrder, LittleEndian};
use std::io;

/// Bits per pixel
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum BitsPerPixel {
	/// 4-bit indexed
	Index4Bit,

	/// 8-bit indexed
	Index8Bit,

	/// 16-bit color
	Color16Bit,

	/// 24-bit color
	Color24Bit,
}

/// `.TIM` file
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct TimFile {
	/// Bits per pixel
	bits_per_pixel: BitsPerPixel,

	/// Color lookup table
	clut: Option<Clut>,
}

impl TimFile {
	/// Header size in bytes
	pub const HEADER_SIZE: usize = 8;
	/// Magic
	pub const MAGIC: u8 = 0x10;
	/// Version
	pub const VERSION: u8 = 0x0;
}

impl TimFile {
	/// Deserializes the clut
	pub fn deserialize<R: io::Read + io::Seek>(reader: &mut R) -> Result<Self, DeserializeError> {
		// Read the whole header
		let mut header_bytes = [0u8; Self::HEADER_SIZE];
		reader
			.read_exact(&mut header_bytes)
			.map_err(DeserializeError::ReadHeader)?;

		let header_bytes = dcb_util::array_split!(&header_bytes,
			magic  :  0x1,
			version:  0x1,
			_unused: [0x2],
			flags  : [0x4],
		);

		// Check the magic and version
		if *header_bytes.magic != Self::MAGIC {
			return Err(DeserializeError::InvalidMagic(*header_bytes.magic));
		}
		if *header_bytes.version != Self::VERSION {
			return Err(DeserializeError::InvalidVersion(*header_bytes.version));
		}

		// Then check the flags
		let flags = LittleEndian::read_u32(header_bytes.flags);
		let bits_per_pixel = match flags & 0b11 {
			0b00 => BitsPerPixel::Index4Bit,
			0b01 => BitsPerPixel::Index8Bit,
			0b10 => BitsPerPixel::Color16Bit,
			0b11 => BitsPerPixel::Color24Bit,
			_ => unreachable!(),
		};
		let color_lookup_table_present = flags & 0b1000 != 0;

		// If there's a color table, read it
		let clut = match color_lookup_table_present {
			true => Some(Clut::deserialize(reader).map_err(DeserializeError::Clut)?),
			false => None,
		};

		// TODO: Read image data

		Ok(Self { bits_per_pixel, clut })
	}
}
