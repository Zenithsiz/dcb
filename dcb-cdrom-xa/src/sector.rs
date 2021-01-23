#![doc(include = "sector.md")]

// Modules
pub mod ecc;
pub mod edc;
pub mod error;
pub mod header;

// Exports
pub use ecc::Ecc;
pub use edc::Edc;
pub use error::FromBytesError;
pub use header::Header;

// Imports
use dcb_bytes::Bytes;
use dcb_util::{array_split, array_split_mut};

/// A CD-ROM/XA Sector
///
/// See the module-level documentation for more details.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Sector {
	/// Header
	pub header: Header,

	/// Data
	pub data: [u8; 2048],
}

impl Bytes for Sector {
	type ByteArray = [u8; 0x930];
	type FromError = FromBytesError;
	type ToError = !;

	fn from_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::FromError> {
		let bytes = array_split!(bytes,
			header: [0x18 ],
			data  : [0x800],
			edc   : [0x4  ],
			ecc   : [0x114],
		);

		let header = Header::from_bytes(bytes.header).map_err(FromBytesError::Header)?;

		/*
		let edc = Edc::from_bytes(bytes.edc).into_ok();
		let mut raw_subheader = [0u8; 0x8];
		header.subheader.to_bytes(&mut raw_subheader).into_ok();
		if !edc.is_valid(&raw_subheader, bytes.data) {
			log::warn!("Found invalid data, attempting correction");
		}
		*/


		Ok(Self { header, data: *bytes.data })
	}

	fn to_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::ToError> {
		let bytes = array_split_mut!(bytes,
			header: [0x18 ],
			data  : [0x800],
			edc   : [0x4  ],
			ecc   : [0x114],
		);

		self.header.to_bytes(bytes.header).into_ok();
		*bytes.data = self.data;

		Ok(())
	}
}
