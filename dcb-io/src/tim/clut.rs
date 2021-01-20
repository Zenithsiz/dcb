//! `.TIM` Color lookup table

// Modules
pub mod error;

use dcb_util::array_split;
// Exports
pub use error::DeserializeError;

// Imports
use byteorder::{ByteOrder, LittleEndian, ReadBytesExt};
use std::{convert::TryFrom, io};

/// Color lookup table
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Clut {
	// TODO: Separate header
	/// x
	x: u16,

	/// y
	y: u16,

	/// Width
	width: u16,

	/// height
	height: u16,

	/// Colors
	colors: Vec<u16>,
}

impl Clut {
	/// Header size in bytes
	pub const HEADER_SIZE: usize = 12;
}

impl Clut {
	/// Deserializes the clut
	pub fn deserialize<R: io::Read>(mut reader: R) -> Result<Self, DeserializeError> {
		// Read the whole header
		let mut header_bytes = [0u8; Self::HEADER_SIZE];
		reader.read_exact(&mut header_bytes).map_err(DeserializeError::ReadHeader)?;

		let header_bytes = array_split!(&header_bytes,
			length: [0x4],
			x     : [0x2],
			y     : [0x2],
			width : [0x2],
			height: [0x2],
		);

		let length = LittleEndian::read_u32(header_bytes.length);
		let x = LittleEndian::read_u16(header_bytes.x);
		let y = LittleEndian::read_u16(header_bytes.y);
		let width = LittleEndian::read_u16(header_bytes.width);
		let height = LittleEndian::read_u16(header_bytes.height);

		// If the length isn't valid, return Err
		let colors_len = usize::from(width) * usize::from(height);
		if usize::try_from(length).expect("Length didn't fit into a `usize`") != Self::HEADER_SIZE + 2 * colors_len {
			return Err(DeserializeError::InvalidLength);
		}

		let mut colors = Vec::with_capacity(colors_len);
		for _ in 0..colors_len {
			let color = reader.read_u16::<LittleEndian>().map_err(DeserializeError::ReadColors)?;
			colors.push(color);
		}

		Ok(Self { x, y, width, height, colors })
	}
}
