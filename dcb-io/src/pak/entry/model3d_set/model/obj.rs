//! Object

// Imports
use byteorder::{ByteOrder, LittleEndian};
use dcb_bytes::Bytes;
use dcb_util::array_split;

/// A `.TMD` object
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Obj {
	/// Vertices position
	pub vertices_pos: u32,

	/// Vertices length
	pub vertices_len: u32,

	/// Normals position
	pub normal_pos: u32,

	/// Normals length
	pub normal_len: u32,

	/// Primitives position
	pub primitive_pos: u32,

	/// Primitives length
	pub primitive_len: u32,

	/// Scale
	pub scale: i32,
}


impl Bytes for Obj {
	type ByteArray = [u8; 0x1c];
	type FromError = !;
	type ToError = !;

	fn from_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::FromError> {
		let bytes = array_split!(bytes,
			vertices_pos : [0x4],
			vertices_len : [0x4],
			normal_pos   : [0x4],
			normal_len   : [0x4],
			primitive_pos: [0x4],
			primitive_len: [0x4],
			scale        : [0x4],
		);

		Ok(Self {
			vertices_pos:  LittleEndian::read_u32(bytes.vertices_pos),
			vertices_len:  LittleEndian::read_u32(bytes.vertices_len),
			normal_pos:    LittleEndian::read_u32(bytes.normal_pos),
			normal_len:    LittleEndian::read_u32(bytes.normal_len),
			primitive_pos: LittleEndian::read_u32(bytes.primitive_pos),
			primitive_len: LittleEndian::read_u32(bytes.primitive_len),
			scale:         LittleEndian::read_i32(bytes.scale),
		})
	}

	fn to_bytes(&self, _bytes: &mut Self::ByteArray) -> Result<(), Self::ToError> {
		todo!()
	}
}
