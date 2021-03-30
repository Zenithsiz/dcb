//! Object

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

dcb_bytes::derive_bytes_split! {Obj,
	vertices_pos : u32 as LittleEndian,
	vertices_len : u32 as LittleEndian,
	normal_pos   : u32 as LittleEndian,
	normal_len   : u32 as LittleEndian,
	primitive_pos: u32 as LittleEndian,
	primitive_len: u32 as LittleEndian,
	scale        : i32 as LittleEndian,
}
