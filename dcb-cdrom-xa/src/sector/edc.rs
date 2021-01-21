#![doc(include = "edc.md")]

// Imports
use byteorder::{ByteOrder, LittleEndian};
use crc::{crc32, Hasher32};
use dcb_bytes::Bytes;

/// Error detection
pub struct Edc {
	/// Crc
	crc: u32,
}

impl Edc {
	/// Polynomial used for CRC
	pub const POLY: u32 = 0b1000_0000_0000_0001_1000_0000_0001_1011;

	/// Checks if `raw_data` is valid.
	#[must_use]
	#[allow(clippy::trivially_copy_pass_by_ref)] // We can more easily calculate a reference to it.
	pub fn is_valid(&self, raw_subheader: &[u8; 0x8], raw_data: &[u8; 0x800]) -> bool {
		let mut digest = crc32::Digest::new(Self::POLY);
		digest.write(raw_subheader);
		digest.write(raw_data);
		let crc = digest.sum32();

		log::warn!("Found crc {:#x}", crc);
		log::warn!("Expected crc {:#x}", self.crc);

		crc == self.crc
	}
}


impl Bytes for Edc {
	type ByteArray = [u8; 4];
	type FromError = !;
	type ToError = !;

	fn from_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::FromError> {
		Ok(Self {
			crc: LittleEndian::read_u32(bytes),
		})
	}

	fn to_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::ToError> {
		LittleEndian::write_u32(bytes, self.crc);
		Ok(())
	}
}
