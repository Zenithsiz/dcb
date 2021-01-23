//! Sector subheader

// Imports
use byteorder::{ByteOrder, LittleEndian};
use dcb_bytes::Bytes;
use dcb_util::{array_split, array_split_mut};

/// The sector sub-header
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct SubHeader {
	/// File
	pub file: u16,

	/// Channel
	pub channel: u16,

	/// Submode
	pub submode: u16,

	/// Data type
	pub data_type: u16,
}

impl Bytes for SubHeader {
	type ByteArray = [u8; 0x8];
	type FromError = !;
	type ToError = !;

	fn from_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::FromError> {
		let bytes = array_split!(bytes,
			file     : [0x2],
			channel  : [0x2],
			submode  : [0x2],
			data_type: [0x2],
		);

		Ok(Self {
			file:      LittleEndian::read_u16(bytes.file),
			channel:   LittleEndian::read_u16(bytes.channel),
			submode:   LittleEndian::read_u16(bytes.submode),
			data_type: LittleEndian::read_u16(bytes.data_type),
		})
	}

	fn to_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::ToError> {
		let bytes = array_split_mut!(bytes,
			file     : [0x2],
			channel  : [0x2],
			submode  : [0x2],
			data_type: [0x2],
		);

		LittleEndian::write_u16(bytes.file, self.file);
		LittleEndian::write_u16(bytes.channel, self.channel);
		LittleEndian::write_u16(bytes.submode, self.submode);
		LittleEndian::write_u16(bytes.data_type, self.data_type);

		Ok(())
	}
}
