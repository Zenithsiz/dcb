//! 2D Animation

// Modules
mod error;
pub mod frame;

// Exports
pub use error::DeserializeError;
pub use frame::Frame;

// Imports
use byteorder::{ByteOrder, LittleEndian};
use dcb_bytes::Bytes;
use dcb_util::{null_ascii_string::NullAsciiString, AsciiStrArr};
use std::io;

/// 2D Animation data
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Animation2d {
	/// File name
	pub file_name: AsciiStrArr<0xb>,

	/// Either `0x1` or `0x64FB54`
	pub unknown0: u32,

	/// Always `0x1`
	pub unknown1: u32,

	/// Either `0x0`, `0x3` or `0xFFFF`
	// Note: Only 3 of them actually have `0x3`, all with
	//       name `1042_0.tim`, E.DRV/1042.PAK, E.DRV/1072.PAK and E.DRV/1204.PAK.
	pub unknown2: u16,

	/// Always `0x0`
	pub unknown3: u16,

	/// Either `0x160` or `0x170`
	pub unknown4: u16,

	/// Either `0x6f`, `0x70`, `0x71`, `0x72` or `0x73`
	pub unknown5: u16,

	/// Either `0x140`, `0x158` or `0x400140`
	// Note: Last value only happens on one, `325_1.tim`, `E.DRV/632.PAK`.
	pub unknown6: u32,

	/// Frames
	pub frames: Vec<Frame>,
}

impl Animation2d {
	/// Header size
	pub const HEADER_SIZE: usize = 36;
}

impl Animation2d {
	/// Deserializes a 2d animation data
	pub fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, DeserializeError> {
		// Read the header
		let mut header_bytes = [0; Self::HEADER_SIZE];
		reader
			.read_exact(&mut header_bytes)
			.map_err(DeserializeError::ReadHeader)?;

		let header_bytes = dcb_util::array_split!(&header_bytes,
			file_name : [0xc],
			unknown0  : [0x4],
			unknown1  : [0x4],
			frames_len: [0x4],
			unknown2  : [0x2],
			unknown3  : [0x2],
			unknown4  : [0x2],
			unknown5  : [0x2],
			unknown6  : [0x4],
		);

		let file_name = header_bytes
			.file_name
			.read_string()
			.map_err(DeserializeError::ParseName)?;
		let frames_len = LittleEndian::read_u32(header_bytes.frames_len);

		let mut frames = vec![];
		for _ in 0..frames_len {
			let mut frame_bytes = [0; 0x14];
			reader
				.read_exact(&mut frame_bytes)
				.map_err(DeserializeError::ReadFrame)?;
			let frame = Frame::deserialize_bytes(&frame_bytes).into_ok();

			frames.push(frame);
		}

		Ok(Self {
			file_name,
			unknown0: LittleEndian::read_u32(header_bytes.unknown0),
			unknown1: LittleEndian::read_u32(header_bytes.unknown1),
			unknown2: LittleEndian::read_u16(header_bytes.unknown2),
			unknown3: LittleEndian::read_u16(header_bytes.unknown3),
			unknown4: LittleEndian::read_u16(header_bytes.unknown4),
			unknown5: LittleEndian::read_u16(header_bytes.unknown5),
			unknown6: LittleEndian::read_u32(header_bytes.unknown6),
			frames,
		})
	}
}
