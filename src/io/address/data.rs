//! File data-only addresses

// Imports
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

// Conversions from and into u64
impl From<Data> for u64 {
	fn from(address: Data) -> Self {
		address.0
	}
}
impl From<u64> for Data {
	fn from(address: u64) -> Self {
		Self(address)
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
		write!(f, "{:x}", u64::from(*self))
	}
}
