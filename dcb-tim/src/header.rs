//! Header

// Modules
pub mod error;

// Export
pub use error::FromBytesError;

// Imports
use byteorder::{ByteOrder, LittleEndian};
use dcb_bytes::Bytes;
use dcb_util::{array_split, array_split_mut};

/// Header
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Header {
	/// Bits per pixel
	pub bbp: BitsPerPixel,

	/// Clut present
	pub clut_present: bool,
}

impl Bytes for Header {
	type ByteArray = [u8; 0x8];
	type FromError = FromBytesError;
	type ToError = !;

	fn from_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::FromError> {
		let bytes = array_split!(bytes,
			tag    :  0x1,
			version:  0x1,
			_unused: [0x2],
			flags  : [0x4],
		);

		// If the magic is wrong, return
		if *bytes.tag != 0x10 {
			return Err(FromBytesError::InvalidTag(*bytes.tag));
		}

		// If the version isn't `0x0`, return
		if *bytes.version != 0x0 {
			return Err(FromBytesError::InvalidVersion(*bytes.version));
		}

		// Else parse the flags
		let flags = LittleEndian::read_u32(bytes.flags);
		let bbp = match flags & 0b11 {
			0b00 => BitsPerPixel::Index4Bit,
			0b01 => BitsPerPixel::Index8Bit,
			0b10 => BitsPerPixel::Color16Bit,
			0b11 => BitsPerPixel::Color24Bit,
			_ => unreachable!(),
		};
		let clut_present = flags & 0b1000 != 0;

		Ok(Self { bbp, clut_present })
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
		let bbp = match self.bbp {
			BitsPerPixel::Index4Bit => 0b00,
			BitsPerPixel::Index8Bit => 0b01,
			BitsPerPixel::Color16Bit => 0b10,
			BitsPerPixel::Color24Bit => 0b11,
		};
		let clut_present = if self.clut_present { 0b1000 } else { 0 };
		LittleEndian::write_u32(bytes.flags, bbp | clut_present);


		Ok(())
	}
}

/// Bits per pixel
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum BitsPerPixel {
	/// 4-bit indexed
	Index4Bit,

	/// 8-bit indexed
	Index8Bit,

	/// 16-bit color
	Color16Bit,

	/// 24-bit color
	Color24Bit,
}
