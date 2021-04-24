#![doc(include = "edc.md")]

// Imports
use byteorder::{ByteOrder, LittleEndian};
use dcb_bytes::Bytes;

/// Error detection
pub struct Edc {
	/// Crc
	pub crc: u32,
}

impl Edc {
	/// Crc Polynomial
	pub const CRC_POLY: u32 = 0xd8018001;
	/// The crc table
	pub const CRC_TABLE: [u32; 256] = Self::crc_table();

	/// Calculates the crc table
	const fn crc_table() -> [u32; 256] {
		let mut table = [0u32; 256];
		let mut n = 0;
		while n < table.len() {
			#[allow(clippy::as_conversions, clippy::cast_possible_truncation)] // `n < 256`
			let mut value = n as u32;
			let mut i = 0usize;
			while i < 8 {
				value = if value & 1 != 0 { Self::CRC_POLY } else { 0 } ^ (value >> 1u32);
				i += 1;
			}
			table[n] = value;
			n += 1;
		}

		table
	}

	/// Checks if `bytes` is valid.
	pub fn is_valid(&self, bytes: &[u8]) -> Result<(), u32> {
		let mut crc = 0;
		#[allow(clippy::as_conversions)]
		for &b in bytes {
			let idx = (crc ^ u32::from(b)) & 0xFF;
			crc = (crc >> 8u32) ^ Self::CRC_TABLE[idx as usize];
		}

		match crc == self.crc {
			true => Ok(()),
			false => Err(crc),
		}
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
