#![doc = include_str!("header.md")]

// Modules
pub mod error;

// Exports
pub use error::DeserializeBytesError;

// Includes
use crate::card::property::CardType;
use byteorder::{ByteOrder, LittleEndian};
use dcb_bytes::Bytes;

/// Card header
pub struct CardHeader {
	/// Card id
	pub id: u16,

	/// Card type
	pub ty: CardType,
}


impl Bytes for CardHeader {
	type ByteArray = [u8; 0x3];
	type DeserializeError = DeserializeBytesError;
	type SerializeError = !;

	fn deserialize_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::DeserializeError> {
		let bytes = dcb_util::array_split!(bytes,
			id: [0x2],
			ty: 0x1,
		);

		let id = LittleEndian::read_u16(bytes.id);
		let ty = CardType::deserialize_bytes(bytes.ty).map_err(DeserializeBytesError::CardType)?;

		Ok(Self { id, ty })
	}

	fn serialize_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::SerializeError> {
		let bytes = dcb_util::array_split_mut!(bytes,
			id: [0x2],
			ty: 0x1,
		);

		LittleEndian::write_u16(bytes.id, self.id);
		self.ty.serialize_bytes(bytes.ty).into_ok();

		Ok(())
	}
}
