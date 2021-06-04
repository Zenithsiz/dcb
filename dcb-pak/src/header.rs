//! Header

// Modules
pub mod error;
pub mod kind;

// Export
pub use error::DeserializeBytesError;
pub use kind::Kind;

// Imports
use byteorder::{ByteOrder, LittleEndian};
use dcb_bytes::Bytes;

/// Header
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Header {
	/// Kind
	pub kind: Kind,

	/// Id
	pub id: u16,

	/// Size of the file
	pub size: u32,
}

impl Bytes for Header {
	type ByteArray = [u8; 0x8];
	type DeserializeError = DeserializeBytesError;
	type SerializeError = !;

	fn deserialize_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::DeserializeError> {
		let bytes = dcb_util::array_split!(bytes,
			file_kind: [0x2],
			file_id  : [0x2],
			size     : [0x4],
		);

		Ok(Self {
			kind: Kind::deserialize_bytes(bytes.file_kind).map_err(DeserializeBytesError::Kind)?,
			id:   LittleEndian::read_u16(bytes.file_id),
			size: LittleEndian::read_u32(bytes.size),
		})
	}

	fn serialize_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::SerializeError> {
		let bytes = dcb_util::array_split_mut!(bytes,
			file_kind: [0x2],
			file_id  : [0x2],
			size     : [0x4],
		);

		self.kind.serialize_bytes(bytes.file_kind).into_ok();
		LittleEndian::write_u16(bytes.file_id, self.id);
		LittleEndian::write_u32(bytes.size, self.size);

		Ok(())
	}
}
