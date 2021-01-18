//! Sector header

// Modules
pub mod error;

// Exports
pub use error::FromBytesError;

// Imports
use super::{SectorAddress, SectorSubHeader};
use dcb_bytes::Bytes;
use dcb_util::{array_split, array_split_mut};

/// The sector header
pub struct SectorHeader {
	/// Sector address
	pub address: SectorAddress,

	/// Subheader
	pub subheader: SectorSubHeader,
}

impl SectorHeader {
	/// Sync's value
	pub const SYNC: [u8; 12] = [0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00];
}

impl Bytes for SectorHeader {
	type ByteArray = [u8; 0x18];
	type FromError = FromBytesError;
	type ToError = !;

	fn from_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::FromError> {
		// Split bytes
		let bytes = array_split!(bytes,
			sync     : [0xc],
			address  : [0x3],
			mode     :  0x1 ,
			subheader: [0x8],
		);

		// Check if the sync is correct
		if bytes.sync != &Self::SYNC {
			return Err(FromBytesError::Sync(*bytes.sync));
		}

		// If we aren't in mode 2, return
		if *bytes.mode != 2 {
			return Err(FromBytesError::Mode(*bytes.mode));
		}

		// Read the address and subheader
		let address = SectorAddress::from_bytes(bytes.address).into_ok();
		let subheader = SectorSubHeader::from_bytes(bytes.subheader).into_ok();

		Ok(Self { address, subheader })
	}

	fn to_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::ToError> {
		// Split bytes
		let bytes = array_split_mut!(bytes,
			sync     : [0xc],
			address  : [0x3],
			mode     :  0x1 ,
			subheader: [0x8],
		);

		*bytes.sync = Self::SYNC;
		self.address.to_bytes(bytes.address).into_ok();
		*bytes.mode = 2;
		self.subheader.to_bytes(bytes.subheader).into_ok();

		Ok(())
	}
}
