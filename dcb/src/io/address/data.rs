//! File data-only addresses

// Imports
use std::convert::TryInto;

use crate::{
	io::address::Real,
	util::{abs_diff, signed_offset},
};

/// A type for defining addresses on the data parts of `.bin` file.
///
/// # Details
/// All addresses of type `Data` will represent the position
/// within *only* the data sections on the file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[derive(derive_more::From, derive_more::Into)]
pub struct Data(u64);

impl Data {
	/// Creates a data address from a `u64`
	#[must_use]
	pub const fn from_u64(address: u64) -> Self {
		Self(address)
	}

	/// Returns this address as a `u64`
	#[must_use]
	pub const fn as_u64(self) -> u64 {
		self.0
	}

	/// Converts this data offset to a real offset
	#[must_use]
	pub const fn to_real(self) -> Real {
		// Get the sector and offset
		let data_sector = self.sector();
		let data_sector_offset = self.offset();

		// Then the real address is just converting the data_sector
		// to a real_sector and adding the header plus the offset
		#[rustfmt::skip]
		Real::from_u64(
			Real::SECTOR_BYTE_SIZE * data_sector + // Base of real sector
			Real::HEADER_BYTE_SIZE                        + // Skip header
			data_sector_offset,                             // Offset inside data sector
		)
	}

	/// Returns the remaining bytes in this data section
	#[must_use]
	pub fn remaining_bytes(self) -> u64 {
		// Note: This can't panic, as we know it's positive.
		(self.to_real().cur_sector_data_section_end() - self.to_real())
			.try_into()
			.expect("Offset was negative")
	}

	/// Returns the sector associated with this address
	#[must_use]
	pub const fn sector(self) -> u64 {
		self.as_u64() / Real::DATA_BYTE_SIZE
	}

	/// Returns the offset into the data section of this address
	#[must_use]
	pub const fn offset(self) -> u64 {
		self.as_u64() % Real::DATA_BYTE_SIZE
	}
}

// Data + Offset
impl std::ops::Add<i64> for Data {
	type Output = Self;

	fn add(self, offset: i64) -> Self {
		Self::from(signed_offset(self.0, offset))
	}
}

// Data += Offset
impl std::ops::AddAssign<i64> for Data {
	fn add_assign(&mut self, offset: i64) {
		*self = *self + offset;
	}
}

// Data + absolute
impl std::ops::Add<u64> for Data {
	type Output = Self;

	fn add(self, absolute: u64) -> Self {
		Self::from(self.0 + absolute)
	}
}

// Data += absolute
impl std::ops::AddAssign<u64> for Data {
	fn add_assign(&mut self, absolute: u64) {
		*self = *self + absolute;
	}
}

// Data - absolute
impl std::ops::Sub<u64> for Data {
	type Output = Self;

	fn sub(self, absolute: u64) -> Self {
		Self::from(self.0 - absolute)
	}
}

// Data -= absolute
impl std::ops::SubAssign<u64> for Data {
	fn sub_assign(&mut self, absolute: u64) {
		*self = *self - absolute;
	}
}

// Data - Data
impl std::ops::Sub<Data> for Data {
	type Output = i64;

	fn sub(self, address: Self) -> i64 {
		abs_diff(self.0, address.0)
	}
}

// Display
impl std::fmt::Display for Data {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:#x}", u64::from(*self))
	}
}

impl From<Data> for Real {
	fn from(data_address: Data) -> Self {
		data_address.to_real()
	}
}
