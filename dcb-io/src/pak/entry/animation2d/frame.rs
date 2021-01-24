//! 2D Animation frame

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

dcb_bytes::derive_bytes_split! {Frame,
	unknown0: u32 as LittleEndian,
	x0      : u8  as LittleEndian,
	x1      : u8  as LittleEndian,
	y0      : u8  as LittleEndian,
	y1      : u8  as LittleEndian,
	width   : u16 as LittleEndian,
	height  : u16 as LittleEndian,
	unknown1: u16 as LittleEndian,
	duration: u16 as LittleEndian,
	unknown2: u16 as LittleEndian,
	unknown3: u16 as LittleEndian,
}
