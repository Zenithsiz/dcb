//! Header

// Modules
pub mod kind;

// Export
pub use kind::Kind;

// Imports
use byteorder::{ByteOrder, LittleEndian};
use dcb_bytes::Bytes;
use dcb_util::{array_split, array_split_mut};

/// Header
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Header {
	/// Kind
	pub kind: Kind,

	/// Id
	pub id: u16,

	/// Size of first file
	pub size: u32,
}

/// Error type for [`Bytes::from_bytes`]
#[derive(Debug, thiserror::Error)]
pub enum FromBytesError {
	/// Unable to parse file kind
	#[error("Unable to parse file kind")]
	Kind(#[source] kind::FromBytesError),
}

impl Bytes for Header {
	type ByteArray = [u8; 0x8];
	type FromError = FromBytesError;
	type ToError = !;

	fn from_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::FromError> {
		let bytes = array_split!(bytes,
			file_kind: [0x2],
			file_id  : [0x2],
			size     : [0x4],
		);

		Ok(Self {
			kind: Kind::from_bytes(bytes.file_kind).map_err(FromBytesError::Kind)?,
			id:   LittleEndian::read_u16(bytes.file_id),
			size:      LittleEndian::read_u32(bytes.size),
		})
	}

	fn to_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::ToError> {
		let bytes = array_split_mut!(bytes,
			file_kind: [0x2],
			file_id  : [0x2],
			size     : [0x4],
		);

		self.kind.to_bytes(bytes.file_kind).into_ok();
		LittleEndian::write_u16(bytes.file_id, self.id);
		LittleEndian::write_u32(bytes.size, self.size);

		Ok(())
	}
}
