//! Header

// Modules
mod error;

// Export
pub use error::DeserializeBytesError;

// Imports
use crate::BitsPerPixel;
use byteorder::{ByteOrder, LittleEndian};
use dcb_bytes::Bytes;

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
	type DeserializeError = DeserializeBytesError;
	type SerializeError = !;

	#[bitmatch::bitmatch]
	fn deserialize_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::DeserializeError> {
		let bytes = dcb_util::array_split!(bytes,
			tag    :  0x1,
			version:  0x1,
			_unused: [0x2],
			flags  : [0x4],
		);

		// If the tag is wrong, return
		if *bytes.tag != 0x10 {
			return Err(DeserializeBytesError::InvalidTag(*bytes.tag));
		}

		// If the version isn't `0x0`, return
		if *bytes.version != 0x0 {
			return Err(DeserializeBytesError::InvalidVersion(*bytes.version));
		}

		// Else parse the flags
		let flags = LittleEndian::read_u32(bytes.flags);
		let (bpp, clut_present) = #[bitmatch]
		match flags {
			"0000_0000_0000_0000_0000_0000_0000_c0bb" => (b, c != 0),
			_ => return Err(DeserializeBytesError::UnknownFlag(flags)),
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

	fn serialize_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::SerializeError> {
		let bytes = dcb_util::array_split_mut!(bytes,
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
