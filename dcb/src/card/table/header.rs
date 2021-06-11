#![doc = include_str!("header.md")]

// Modules
mod error;

// Exports
pub use error::DeserializeBytesError;

// Imports
use byteorder::{ByteOrder, LittleEndian};
use dcb_bytes::Bytes;

/// The header
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Header {
	/// Number of digimon
	pub digimons_len: u16,

	/// Number of items
	pub items_len: u8,

	/// Number of digivolves
	pub digivolves_len: u8,
}

impl Header {
	/// Magic of this header.
	/// = "0ACD"
	pub const MAGIC: [u8; 4] = *b"0ACD";
}

impl Bytes for Header {
	type ByteArray = [u8; 0x8];
	type DeserializeError = DeserializeBytesError;
	type SerializeError = !;

	fn deserialize_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::DeserializeError> {
		let bytes = dcb_util::array_split!(bytes,
			magic: [0x4],
			digimons_len: [0x2],
			items_len: 1,
			digivolves_len: 1,
		);

		if *bytes.magic != Self::MAGIC {
			return Err(DeserializeBytesError::Magic { magic: *bytes.magic });
		}

		let digimons_len = LittleEndian::read_u16(bytes.digimons_len);
		let items_len = *bytes.items_len;
		let digivolves_len = *bytes.digivolves_len;

		Ok(Self {
			digimons_len,
			items_len,
			digivolves_len,
		})
	}

	fn serialize_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::SerializeError> {
		let bytes = dcb_util::array_split_mut!(bytes,
			magic: [0x4],
			digimons_len: [0x2],
			items_len: 1,
			digivolves_len: 1,
		);

		*bytes.magic = Self::MAGIC;
		LittleEndian::write_u16(bytes.digimons_len, self.digimons_len);
		*bytes.items_len = self.items_len;
		*bytes.digivolves_len = self.digivolves_len;

		Ok(())
	}
}
