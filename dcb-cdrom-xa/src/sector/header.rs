//! Sector header

// Modules
pub mod address;
pub mod error;
pub mod subheader;

// Exports
pub use address::Address;
pub use error::{FromBytesError, ToBytesError};
pub use subheader::SubHeader;

// Imports
use dcb_bytes::Bytes;
use dcb_util::{array_split, array_split_mut};

/// The sector header
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Header {
	/// Sector address
	pub address: Address,

	/// Subheader
	// Note: Repeated twice
	pub subheader: SubHeader,
}

impl Header {
	/// Sync's value
	pub const SYNC: [u8; 12] = [0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00];
}

impl Bytes for Header {
	type ByteArray = [u8; 0x18];
	type FromError = FromBytesError;
	type ToError = ToBytesError;

	fn from_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::FromError> {
		let bytes = array_split!(bytes,
			sync      : [0xc],
			address   : [0x3],
			mode      :  0x1 ,
			subheader1: [0x4],
			subheader2: [0x4],
		);

		// Check if the sync is correct
		if bytes.sync != &Self::SYNC {
			return Err(FromBytesError::WrongSync(*bytes.sync));
		}

		// If we aren't in mode 2, return
		if *bytes.mode != 2 {
			return Err(FromBytesError::InvalidMode(*bytes.mode));
		}

		// Read the two sub-headers
		let subheader1 = SubHeader::from_bytes(bytes.subheader1).map_err(FromBytesError::SubHeader)?;
		let subheader2 = SubHeader::from_bytes(bytes.subheader2).map_err(FromBytesError::SubHeader)?;

		if subheader1 != subheader2 {
			return Err(FromBytesError::DifferentSubHeaders(subheader1, subheader2));
		}

		// Read the address
		let address = Address::from_bytes(bytes.address).map_err(FromBytesError::Address)?;


		Ok(Self {
			address,
			subheader: subheader1,
		})
	}

	fn to_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::ToError> {
		let bytes = array_split_mut!(bytes,
			sync      : [0xc],
			address   : [0x3],
			mode      :  0x1 ,
			subheader1: [0x4],
			subheader2: [0x4],
		);

		*bytes.sync = Self::SYNC;
		self.address.to_bytes(bytes.address).map_err(ToBytesError::Address)?;
		*bytes.mode = 2;
		self.subheader
			.to_bytes(bytes.subheader1)
			.map_err(ToBytesError::SubHeader)?;
		self.subheader
			.to_bytes(bytes.subheader2)
			.map_err(ToBytesError::SubHeader)?;

		Ok(())
	}
}
