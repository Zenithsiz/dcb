//! Header

// Modules
pub mod error;

// Export
pub use error::FromBytesError;

// Imports
use crate::BitsPerPixel;
use byteorder::{ByteOrder, LittleEndian};
use dcb_bytes::Bytes;
use dcb_util::{array_split, array_split_mut};

/// Header
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Header {
	/// Bits per pixel
	pub bpp: BitsPerPixel,

	/// Clut present
	pub clut_present: bool,
}

impl Bytes for Header {
	type ByteArray = [u8; 0x8];
	type FromError = FromBytesError;
	type ToError = !;

	#[bitmatch::bitmatch]
	fn from_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::FromError> {
		let bytes = array_split!(bytes,
			tag    :  0x1,
			version:  0x1,
			_unused: [0x2],
			flags  : [0x4],
		);

		// If the tag is wrong, return
		if *bytes.tag != 0x10 {
			return Err(FromBytesError::InvalidTag(*bytes.tag));
		}

		// If the version isn't `0x0`, return
		if *bytes.version != 0x0 {
			return Err(FromBytesError::InvalidVersion(*bytes.version));
		}

		// Else parse the flags
		let flags = LittleEndian::read_u32(bytes.flags);
		let (bpp, clut_present) = #[bitmatch]
		match flags {
			"0000_0000_0000_0000_0000_0000_0000_c0bb" => (b, c != 0),
			_ => return Err(FromBytesError::UnknownFlag(flags)),
		};
		let bpp = match bpp {
			0b00 => BitsPerPixel::Index4Bit,
			0b01 => BitsPerPixel::Index8Bit,
			0b10 => BitsPerPixel::Color16Bit,
			0b11 => BitsPerPixel::Color24Bit,
			_ => unreachable!(),
		};

		Ok(Self { bpp, clut_present })
	}

	fn to_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::ToError> {
		let bytes = array_split_mut!(bytes,
			tag    :  0x1,
			version:  0x1,
			_unused: [0x2],
			flags  : [0x4],
		);

		// Write the tag and version
		*bytes.tag = 0x10;
		*bytes.version = 0x0;

		// Then write the flags
		let bpp = match self.bpp {
			BitsPerPixel::Index4Bit => 0b00,
			BitsPerPixel::Index8Bit => 0b01,
			BitsPerPixel::Color16Bit => 0b10,
			BitsPerPixel::Color24Bit => 0b11,
		};
		let clut_present = if self.clut_present { 0b1000 } else { 0 };
		LittleEndian::write_u32(bytes.flags, bpp | clut_present);


		Ok(())
	}
}
