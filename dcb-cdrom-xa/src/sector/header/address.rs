//! Sector address

// Module
pub mod error;

// Exports
pub use error::{FromBytesError, FromSectorPosError, ToBytesError};

// Imports
use dcb_util::{array_split, array_split_mut, BcdU8};
use std::{convert::TryFrom, ops::Range};

/// Sector address
// TODO: All of these are BCD, read and write them them as such.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Address {
	/// Minutes
	pub min: u8,

	/// Seconds
	pub sec: u8,

	/// Block
	pub block: u8,
}

impl Address {
	/// Block range
	pub const BLOCK_RANGE: Range<u8> = 0..75;
	/// Seconds range
	pub const SECS_RANGE: Range<u8> = 0..60;

	/// Creates a new sector given a position
	///
	/// Starts the first sector at 2 seconds.
	pub fn from_sector_pos(sector_pos: usize) -> Result<Self, FromSectorPosError> {
		let block = u8::try_from(sector_pos % 75).expect("Must fit");
		let total_secs = sector_pos / 75;
		let sec = u8::try_from(total_secs % 60).expect("Must fit");
		#[allow(clippy::map_err_ignore)] // We want to ignore the error here, only one way for it to fail
		let min = u8::try_from(total_secs / 60).map_err(|_| FromSectorPosError::TooLarge(sector_pos))?;

		Ok(Self { min, sec, block })
	}
}

#[allow(clippy::ptr_offset_with_cast)]
impl dcb_bytes::Bytes for Address {
	type ByteArray = [u8; 3];
	type FromError = FromBytesError;
	type ToError = ToBytesError;

	fn from_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::FromError> {
		let bytes = array_split!(bytes,
			min  : 0x1,
			sec  : 0x1,
			block: 0x1,
		);

		let min = BcdU8(*bytes.min)
			.to_u8()
			.ok_or(FromBytesError::InvalidMinute(*bytes.min))?;
		let sec = BcdU8(*bytes.sec)
			.to_u8()
			.ok_or(FromBytesError::InvalidSecond(*bytes.sec))?;
		let block = BcdU8(*bytes.block)
			.to_u8()
			.ok_or(FromBytesError::InvalidBlock(*bytes.block))?;

		if !Self::SECS_RANGE.contains(&sec) {
			return Err(FromBytesError::OutOfRangeSecond(sec));
		}
		if !Self::BLOCK_RANGE.contains(&block) {
			return Err(FromBytesError::OutOfRangeBlock(block));
		}

		Ok(Self { min, sec, block })
	}

	fn to_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::ToError> {
		let bytes = array_split_mut!(bytes,
			min  : 0x1,
			sec  : 0x1,
			block: 0x1,
		);

		if !Self::SECS_RANGE.contains(&self.sec) {
			return Err(ToBytesError::OutOfRangeSecond(self.sec));
		}
		if !Self::BLOCK_RANGE.contains(&self.block) {
			return Err(ToBytesError::OutOfRangeBlock(self.block));
		}

		let min = BcdU8::from_u8(self.min)
			.ok_or(ToBytesError::OutOfRangeMinute(self.min))?
			.0;
		let sec = BcdU8::from_u8(self.sec)
			.ok_or(ToBytesError::OutOfRangeSecond(self.sec))?
			.0;
		let block = BcdU8::from_u8(self.block)
			.ok_or(ToBytesError::OutOfRangeBlock(self.block))?
			.0;

		*bytes.min = min;
		*bytes.sec = sec;
		*bytes.block = block;

		Ok(())
	}
}
