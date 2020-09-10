//! File real addresses

// Imports
use crate::{
	io::address::Data,
	util::{abs_diff, signed_offset},
};

/// A type for defining addresses on the `.bin` file.
///
/// All real addresses will depict the actual position
/// within the game file, including headers from the `.bin` file format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[derive(derive_more::From, derive_more::Into)]
pub struct Real(u64);

/// Error type for [`Real::to_data`]
#[derive(PartialEq, Eq, Clone, Copy, Debug, thiserror::Error)]
pub enum ToDataError {
	/// Occurs when the Real is outside of the data section of the sector
	#[error("Unable to convert real address {} to a data address, as it was not in the data section", .0)]
	OutsideDataSection(Real),
}

// Constants
impl Real {
	/// The number of bytes the data section takes up in the sector
	pub const DATA_BYTE_SIZE: u64 = 2048;
	/// The end of the data section (one-past)
	pub const DATA_END: u64 = Self::HEADER_BYTE_SIZE + Self::DATA_BYTE_SIZE;
	/// The range of the data section
	pub const DATA_RANGE: std::ops::Range<u64> = Self::DATA_START..Self::DATA_END;
	/// The start of the data section
	pub const DATA_START: u64 = Self::HEADER_BYTE_SIZE;
	/// The number of bytes the footer takes up in the sector
	pub const FOOTER_BYTE_SIZE: u64 = 280;
	/// The number of bytes the header takes up in the sector
	pub const HEADER_BYTE_SIZE: u64 = 24;
	/// The number of bytes within a whole sector
	pub const SECTOR_BYTE_SIZE: u64 = 2352;
}

impl Real {
	/// Creates a real address from a `u64`
	#[must_use]
	pub const fn from_u64(address: u64) -> Self {
		Self(address)
	}

	/// Returns this address as a `u64`
	#[must_use]
	pub const fn as_u64(self) -> u64 {
		self.0
	}

	/// Converts this real sector into a data sector
	pub const fn try_to_data(self) -> Result<Data, ToDataError> {
		// If the real address isn't in the data section, then return err
		if !self.in_data_section() {
			return Err(ToDataError::OutsideDataSection(self));
		}

		// Else get the sector and offset
		let real_sector = self.sector();
		let real_sector_offset = self.offset();

		// The data address is just converting the real_sector
		// to a data_sector and subtracting the header from the
		// real offset to get the data offset
		#[rustfmt::skip]
		Ok(Data::from_u64(
			Self::SECTOR_BYTE_SIZE * real_sector + // Base of data sector
			real_sector_offset - Self::HEADER_BYTE_SIZE,    // Data offset (skipping header)
		))
	}

	/// Returns the real sector associated with this address
	#[must_use]
	pub const fn sector(self) -> u64 {
		self.as_u64() / Self::SECTOR_BYTE_SIZE
	}

	/// Returns the offset into the sector of this address
	#[must_use]
	pub const fn offset(self) -> u64 {
		self.as_u64() % Self::SECTOR_BYTE_SIZE
	}

	/// Returns the address of the end of the data section in this sector.
	#[must_use]
	pub const fn data_section_end(self) -> Self {
		// Get the sector
		let real_sector = self.sector();

		// The end of the real data section is after the header and data sections
		#[rustfmt::skip]
		Self::from_u64(
			Self::SECTOR_BYTE_SIZE * real_sector + // Beginning of sector
			Self::HEADER_BYTE_SIZE               + // Skip Header
			Self::  DATA_BYTE_SIZE, // Skip Data
		)
	}

	/// Checks if this address is within the real data section
	#[must_use]
	pub const fn in_data_section(self) -> bool {
		// If our offset is within the data range
		// TODO: Replace with `Self::DATA_RANGE.contains(&self.offset())` once it's `const`.
		let offset = self.offset();
		offset >= Self::DATA_RANGE.start && offset < Self::DATA_RANGE.end
	}
}

// Real + Offset
impl std::ops::Add<i64> for Real {
	type Output = Self;

	fn add(self, offset: i64) -> Self {
		Self::from(signed_offset(self.into(), offset))
	}
}

// Real += Offset
impl std::ops::AddAssign<i64> for Real {
	fn add_assign(&mut self, offset: i64) {
		*self = *self + offset;
	}
}

// Real - Offset
impl std::ops::Sub<i64> for Real {
	type Output = Self;

	fn sub(self, offset: i64) -> Self {
		if offset == i64::MIN {
			panic!("Unable to offset `i64::MIN`")
		}
		Self::from(signed_offset(self.into(), -offset))
	}
}

// Real += Offset
impl std::ops::SubAssign<i64> for Real {
	fn sub_assign(&mut self, offset: i64) {
		*self = *self - offset;
	}
}

// Real - Real
impl std::ops::Sub<Real> for Real {
	type Output = i64;

	fn sub(self, address: Self) -> i64 {
		abs_diff(self.as_u64(), address.as_u64())
	}
}

// Display
impl std::fmt::Display for Real {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:x}", self.as_u64())
	}
}

// Real -> Data
impl std::convert::TryFrom<Real> for Data {
	type Error = ToDataError;

	fn try_from(real_address: Real) -> Result<Self, Self::Error> {
		real_address.try_to_data()
	}
}
