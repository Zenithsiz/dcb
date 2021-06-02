//! Header

// Imports
use byteorder::{ByteOrder, LittleEndian};
use dcb_bytes::Bytes;
use dcb_util::{array_split, array_split_mut};

/// Header
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Header {
	/// Position
	pub pos: [u16; 2],

	/// Size
	pub size: [u16; 2],
}

impl Header {
	/// Returns the total number of colors
	#[must_use]
	pub fn colors_len(self) -> usize {
		usize::from(self.size[0]) * usize::from(self.size[1])
	}
}

impl Bytes for Header {
	type ByteArray = [u8; 0xc];
	type DeserializeError = !;
	type SerializeError = !;

	fn deserialize_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::DeserializeError> {
		let bytes = array_split!(bytes,
			_length: [0x4],
			x      : [0x2],
			y      : [0x2],
			width  : [0x2],
			height : [0x2],
		);

		let x = LittleEndian::read_u16(bytes.x);
		let y = LittleEndian::read_u16(bytes.y);
		let width = LittleEndian::read_u16(bytes.width);
		let height = LittleEndian::read_u16(bytes.height);


		Ok(Self {
			pos:  [x, y],
			size: [width, height],
		})
	}

	fn serialize_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::SerializeError> {
		let bytes = array_split_mut!(bytes,
			length: [0x4],
			x     : [0x2],
			y     : [0x2],
			width : [0x2],
			height: [0x2],
		);

		let [x, y] = self.pos;
		let [width, height] = self.size;
		let length = u32::from(width) * u32::from(height) * 2;
		LittleEndian::write_u32(bytes.length, length);
		LittleEndian::write_u16(bytes.x, x);
		LittleEndian::write_u16(bytes.y, y);
		LittleEndian::write_u16(bytes.width, width);
		LittleEndian::write_u16(bytes.height, height);

		Ok(())
	}
}
