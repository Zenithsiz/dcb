//! A CD-ROM/XA Sector
//!
//! Each sector consists of `0x930` bytes, with a `0x18` byte header,
//! a `0x800` byte data section and a `0x118` footer for error checking
//! and correction.
//!
//! Currently, while the header is mostly parsed, the error correction checking
//! is not done, neither on reading nor writing. Due to this, it may not be suitable
//! to use the output for an actual CD-ROM sector.

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

/// A CD-ROM/XA Sector
///
/// See the module-level documentation for more details.
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
