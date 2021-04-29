//! Instruction position

// Imports
use int_conv::{SignExtended, Signed, Truncated};
use std::{convert::TryFrom, ops};

/// An instruction position
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(derive_more::Display)]
#[display(fmt = "{_0:#x}")]
pub struct Pos(pub u32);

impl Pos {
	/// Calculated the offset between two positions
	///
	/// # Panics
	/// Panics if the result would be negative.
	#[must_use]
	pub fn offset_from(self, start_pos: Self) -> usize {
		usize::try_from(self - start_pos).expect("Negative offset")
	}
}

// Alignment
impl Pos {
	/// Returns if this memory address is aligned to `align`
	#[must_use]
	pub fn is_aligned_to(self, align: usize) -> bool {
		// We're definitely not aligned to anything above
		// `u32::MAX`
		let align = match u32::try_from(align) {
			Ok(align) => align,
			Err(_) => return false,
		};

		self.0 % align == 0
	}

	/// Returns if this memory address is aligned to a word
	#[must_use]
	pub fn is_word_aligned(self) -> bool {
		self.is_aligned_to(4)
	}

	/// Returns if this memory address is aligned to a half-word
	#[must_use]
	pub fn is_half_word_aligned(self) -> bool {
		self.is_aligned_to(2)
	}
}


// `Pos + u32 = Pos`
impl ops::Add<u32> for Pos {
	type Output = Self;

	fn add(self, rhs: u32) -> Self::Output {
		Self(self.0.wrapping_add(rhs))
	}
}

// `Pos + i32 = Pos`
impl ops::Add<i32> for Pos {
	type Output = Self;

	fn add(self, rhs: i32) -> Self::Output {
		Self((self.0.as_signed().wrapping_add(rhs)).as_unsigned())
	}
}

// `Pos + i64 = Pos`
impl ops::Add<i64> for Pos {
	type Output = Self;

	fn add(self, rhs: i64) -> Self::Output {
		Self(
			(self.0.as_signed().sign_extended::<i64>().wrapping_add(rhs))
				.truncated::<i32>()
				.as_unsigned(),
		)
	}
}

// `Pos + usize = Pos`
impl ops::Add<usize> for Pos {
	type Output = Self;

	fn add(self, rhs: usize) -> Self::Output {
		self + u32::try_from(rhs).expect("Value was too large")
	}
}


// `Pos - u32 = Pos`
impl ops::Sub<u32> for Pos {
	type Output = Self;

	fn sub(self, rhs: u32) -> Self::Output {
		Self(self.0.wrapping_sub(rhs))
	}
}

// `Pos - Pos = i64`
impl ops::Sub<Pos> for Pos {
	type Output = i64;

	fn sub(self, rhs: Self) -> Self::Output {
		self.0.as_signed().sign_extended::<i64>() - rhs.0.as_signed().sign_extended::<i64>()
	}
}

// `Pos & u32 = Pos`
impl ops::BitAnd<u32> for Pos {
	type Output = Self;

	fn bitand(self, rhs: u32) -> Self::Output {
		Self(self.0 & rhs)
	}
}

// `Pos += T <=> Pos = Pos + T`
impl<T> ops::AddAssign<T> for Pos
where
	Pos: ops::Add<T, Output = Self>,
{
	fn add_assign(&mut self, rhs: T) {
		*self = *self + rhs;
	}
}

// `&Pos + T`
impl<T> ops::Add<T> for &'_ Pos
where
	Pos: ops::Add<T>,
{
	type Output = <Pos as ops::Add<T>>::Output;

	fn add(self, rhs: T) -> Self::Output {
		<Pos as ops::Add<T>>::add(*self, rhs)
	}
}

// `&Pos - T`
impl<T> ops::Sub<T> for &'_ Pos
where
	Pos: ops::Sub<T>,
{
	type Output = <Pos as ops::Sub<T>>::Output;

	fn sub(self, rhs: T) -> Self::Output {
		<Pos as ops::Sub<T>>::sub(*self, rhs)
	}
}
