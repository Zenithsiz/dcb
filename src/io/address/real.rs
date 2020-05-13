//! File real addresses

/// A type for defining addresses on the `.bin` file.
///
/// All real addresses will depict the actual position
/// within the game file, including headers from the `.bin` file format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[derive(derive_more::From, derive_more::Into)]
pub struct Real(u64);

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
	/// Returns the real sector associated with this address
	#[must_use]
	#[allow(clippy::integer_division)] // We want to get the whole division
	pub fn sector(self) -> u64 {
		u64::from(self) / Self::SECTOR_BYTE_SIZE
	}

	/// Returns the offset into the sector of this address
	#[must_use]
	pub fn offset(self) -> u64 {
		u64::from(self) % Self::SECTOR_BYTE_SIZE
	}

	/// Returns the address of the end of the data section in this sector.
	#[must_use]
	pub fn data_section_end(self) -> Self {
		// Get the sector
		let real_sector = self.sector();

		// The end of the real data section is after the header and data sections
		#[rustfmt::skip]
		Self::from(
			Self::SECTOR_BYTE_SIZE * real_sector + // Beginning of sector
			Self::HEADER_BYTE_SIZE               + // Skip Header
			Self::  DATA_BYTE_SIZE, // Skip Data
		)
	}

	/// Checks if this address is within the real data section
	#[must_use]
	pub fn in_data_section(self) -> bool {
		// If our offset is within the data range
		Self::DATA_RANGE.contains(&self.offset())
	}
}

// Real + Offset
impl std::ops::Add<i64> for Real {
	type Output = Self;

	fn add(self, offset: i64) -> Self {
		Self::from((u64::from(self) as i64 + offset) as u64)
	}
}

// Real += Offset
impl std::ops::AddAssign<i64> for Real {
	fn add_assign(&mut self, offset: i64) {
		*self = *self + offset;
	}
}

// Real - Real
impl std::ops::Sub<Real> for Real {
	type Output = i64;

	fn sub(self, address: Self) -> i64 {
		u64::from(self) as i64 - u64::from(address) as i64
	}
}

// Display
impl std::fmt::Display for Real {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:x}", u64::from(*self))
	}
}
