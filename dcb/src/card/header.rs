#![doc(include = "header.md")]

// Modules
pub mod error;

// Exports
pub use error::FromBytesError;

// Includes
use crate::card::property::CardType;
use byteorder::{ByteOrder, LittleEndian};
use dcb_bytes::Bytes;
use dcb_util::{array_split, array_split_mut};

/// Card header
pub struct CardHeader {
	/// Card id
	pub id: u16,

	/// Card type
	pub ty: CardType,
}


impl Bytes for CardHeader {
	type ByteArray = [u8; 0x3];
	type FromError = FromBytesError;
	type ToError = !;

	fn from_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::FromError> {
		let bytes = array_split!(bytes,
			id: [0x2],
			ty: 0x1,
		);

		let id = LittleEndian::read_u16(bytes.id);
		let ty = CardType::from_bytes(bytes.ty).map_err(FromBytesError::CardType)?;

		Ok(Self { id, ty })
	}

	fn to_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::ToError> {
		let bytes = array_split_mut!(bytes,
			id: [0x2],
			ty: 0x1,
		);

		LittleEndian::write_u16(bytes.id, self.id);
		self.ty.to_bytes(bytes.ty).into_ok();

		Ok(())
	}
}
