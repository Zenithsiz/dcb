//! Header

// Imports
use byteorder::{ByteOrder, LittleEndian};
use dcb_bytes::Bytes;
use dcb_util::{array_split, array_split_mut};
use std::convert::TryInto;

/// Header
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Header {
	/// Length
	pub length: usize,

	/// Position
	pub pos: [u16; 2],

	/// Size
	pub size: [u16; 2],
}

impl Bytes for Header {
	type ByteArray = [u8; 0xc];
	type FromError = !;
	type ToError = !;

	fn from_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::FromError> {
		let bytes = array_split!(bytes,
			length: [0x4],
			x     : [0x2],
			y     : [0x2],
			width : [0x2],
			height: [0x2],
		);

		let length = LittleEndian::read_u32(bytes.length);
		let x = LittleEndian::read_u16(bytes.x);
		let y = LittleEndian::read_u16(bytes.y);
		let width = LittleEndian::read_u16(bytes.width);
		let height = LittleEndian::read_u16(bytes.height);

		let length = length.try_into().expect("Unable to get `u32` as `usize`");

		Ok(Self {
			length,
			pos: [x, y],
			size: [width, height],
		})
	}

	fn to_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::ToError> {
		let bytes = array_split_mut!(bytes,
			length: [0x4],
			x     : [0x2],
			y     : [0x2],
			width : [0x2],
			height: [0x2],
		);

		let [x, y] = self.pos;
		let [width, height] = self.size;
		let length = self.length.try_into().expect("Unable to get `usize` as `u64`");
		LittleEndian::write_u32(bytes.length, length);
		LittleEndian::write_u16(bytes.x, x);
		LittleEndian::write_u16(bytes.y, y);
		LittleEndian::write_u16(bytes.width, width);
		LittleEndian::write_u16(bytes.height, height);

		Ok(())
	}
}
