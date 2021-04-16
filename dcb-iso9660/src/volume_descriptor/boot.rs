//! Boot volume descriptor

// Modules
pub mod error;

// Exports
pub use error::FromBytesError;

// Imports
use crate::StrArrA;
use dcb_bytes::Bytes;
use dcb_util::array_split;

/// Primary volume descriptor
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct BootRecordVolumeDescriptor {
	/// System Id
	pub system_id: StrArrA<0x20>,

	/// Boot identifier
	pub boot_id: StrArrA<0x20>,

	/// Data
	pub data: [u8; 0x7b9],
}

impl Bytes for BootRecordVolumeDescriptor {
	type ByteArray = [u8; 0x7f9];
	type FromError = FromBytesError;
	type ToError = !;

	fn from_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::FromError> {
		let bytes = array_split!(bytes,
			system_id: [0x20],
			boot_id  : [0x20],
			data     : [0x7b9],
		);

		// Parse both ids
		let system_id = StrArrA::from_bytes(bytes.system_id).map_err(FromBytesError::SystemId)?;
		let boot_id = StrArrA::from_bytes(bytes.boot_id).map_err(FromBytesError::BootId)?;

		Ok(Self {
			system_id,
			boot_id,
			data: *bytes.data,
		})
	}

	fn to_bytes(&self, _bytes: &mut Self::ByteArray) -> Result<(), Self::ToError> {
		todo!()
	}
}
