//! Instruction position
// TODO: More implementations for `Pos`

// Imports
use int_conv::Signed;
use std::{fmt, ops};

/// An instruction position
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(ref_cast::RefCast)]
#[serde(transparent)]
#[repr(transparent)]
pub struct Pos(pub u32);

impl fmt::LowerHex for Pos {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		<u32 as fmt::LowerHex>::fmt(&self.0, f)
	}
}

impl ops::Sub<u32> for Pos {
	type Output = Self;

	fn sub(self, rhs: u32) -> Self::Output {
		Self(self.0 - rhs)
	}
}

impl ops::Add<u32> for Pos {
	type Output = Self;

	fn add(self, rhs: u32) -> Self::Output {
		Self(self.0 + rhs)
	}
}

impl ops::Add<i32> for Pos {
	type Output = Self;

	fn add(self, rhs: i32) -> Self::Output {
		Self((self.0.as_signed() + rhs).as_unsigned())
	}
}

impl ops::BitAnd<u32> for Pos {
	type Output = Self;

	fn bitand(self, rhs: u32) -> Self::Output {
		Self(self.0 & rhs)
	}
}

impl<'a, T> ops::Sub<T> for &'_ Pos
where
	Pos: ops::Sub<T, Output = Pos>,
{
	type Output = Pos;

	fn sub(self, rhs: T) -> Self::Output {
		<Pos as ops::Sub<T>>::sub(Pos(self.0), rhs)
	}
}

impl<'a, T> ops::Add<T> for &'_ Pos
where
	Pos: ops::Add<T, Output = Pos>,
{
	type Output = Pos;

	fn add(self, rhs: T) -> Self::Output {
		<Pos as ops::Add<T>>::add(Pos(self.0), rhs)
	}
}
