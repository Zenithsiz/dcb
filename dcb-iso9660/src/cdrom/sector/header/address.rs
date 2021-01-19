//! Sector address

// Imports
use dcb_bytes::Bytes;
use dcb_util::{array_split, array_split_mut};

/// Sector address
pub struct Address {
	/// Minutes
	pub min: u8,

	/// Seconds
	pub sec: u8,

	/// Block
	pub block: u8,
}

impl Bytes for Address {
	type ByteArray = [u8; 0x3];
	type FromError = !;
	type ToError = !;

	fn from_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::FromError> {
		let bytes = array_split!(bytes,
			min  : 0x1,
			sec  : 0x1,
			block: 0x1,
		);

		Ok(Self {
			min:   *bytes.min,
			sec:   *bytes.sec,
			block: *bytes.block,
		})
	}

	fn to_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::ToError> {
		let bytes = array_split_mut!(bytes,
			min  : 0x1,
			sec  : 0x1,
			block: 0x1,
		);

		*bytes.min = self.min;
		*bytes.sec = self.sec;
		*bytes.block = self.block;

		Ok(())
	}
}
