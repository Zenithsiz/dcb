//! A game file sector

// Modules
pub mod address;
pub mod error;
pub mod header;
pub mod subheader;

// Exports
pub use address::SectorAddress;
pub use error::FromBytesError;
pub use header::SectorHeader;
pub use subheader::SectorSubHeader;

// Imports
use dcb_bytes::Bytes;
use dcb_util::{array_split, array_split_mut};

/// A game file sector, `0x930` bytes.
pub struct Sector {
	/// Header
	pub header: SectorHeader,

	/// Data
	pub data: [u8; 2048],
}

impl Bytes for Sector {
	type ByteArray = [u8; 0x930];
	type FromError = FromBytesError;
	type ToError = !;

	fn from_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::FromError> {
		// Split bytes
		let bytes = array_split!(bytes,
			header: [0x18 ],
			data  : [0x800],
			// TODO: Check errors with sector
			_error: [0x118],
		);

		let header = SectorHeader::from_bytes(bytes.header).map_err(FromBytesError::Header)?;
		Ok(Self { header, data: *bytes.data })
	}

	fn to_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::ToError> {
		// Split bytes
		let bytes = array_split_mut!(bytes,
			header: [0x18 ],
			data  : [0x800],
			// TODO: Write error correction to this sector
			_error: [0x118],
		);

		self.header.to_bytes(bytes.header).into_ok();
		*bytes.data = self.data;

		Ok(())
	}
}
