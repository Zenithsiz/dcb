//! 2D Animation frame

// Imports
use byteorder::{ByteOrder, LittleEndian};
use dcb_bytes::Bytes;
use dcb_util::array_split;

/// 2D Animation frame
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Frame {
	/// TODO
	pub unknown0: u32,

	/// Starting x position
	pub x0: u8,

	/// Ending x position
	pub x1: u8,

	/// Starting y position
	pub y0: u8,

	/// Ending y position
	pub y1: u8,

	/// Width
	pub width: u16,

	/// Height
	pub height: u16,

	/// Either `0x0`, `0x1`, `0x2`, `0x3` or `0xFFFF`
	pub unknown1: u16,

	/// Frame duration
	pub duration: u16,

	/// Some king of flags? All values are pretty high
	pub unknown2: u16,

	/// Always 0
	pub unknown3: u16,
}

impl Bytes for Frame {
	type ByteArray = [u8; 0x14];
	type FromError = !;
	type ToError = !;

	fn from_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::FromError> {
		let bytes = array_split!(bytes,
			unknown0: [0x4],
			x0      :  0x1,
			x1      :  0x1,
			y0      :  0x1,
			y1      :  0x1,
			width   : [0x2],
			height  : [0x2],
			unknown1: [0x2],
			duration: [0x2],
			unknown2: [0x2],
			unknown3: [0x2],
		);
		Ok(Self {
			unknown0: LittleEndian::read_u32(bytes.unknown0),
			x0:       *bytes.x0,
			x1:       *bytes.x1,
			y0:       *bytes.y0,
			y1:       *bytes.y1,
			width:    LittleEndian::read_u16(bytes.width),
			height:   LittleEndian::read_u16(bytes.height),
			unknown1: LittleEndian::read_u16(bytes.unknown1),
			duration: LittleEndian::read_u16(bytes.duration),
			unknown2: LittleEndian::read_u16(bytes.unknown2),
			unknown3: LittleEndian::read_u16(bytes.unknown3),
		})
	}

	fn to_bytes(&self, _bytes: &mut Self::ByteArray) -> Result<(), Self::ToError> {
		todo!()
	}
}
