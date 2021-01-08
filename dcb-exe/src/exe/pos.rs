//! Instruction position
// TODO: More implementations for `Pos`

// Imports
use int_conv::{SignExtended, Signed};
use std::{convert::TryFrom, fmt, ops};

/// An instruction position
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, Debug)]
#[derive(ref_cast::RefCast)]
#[derive(derive_more::Display)]
#[display(fmt = "{_0:#x?}")]
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

impl ops::Sub<Pos> for Pos {
	type Output = i64;

	fn sub(self, rhs: Self) -> Self::Output {
		self.0.as_signed().sign_extended::<i64>() - rhs.0.as_signed().sign_extended::<i64>()
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

impl ops::Add<i16> for Pos {
	type Output = Self;

	fn add(self, rhs: i16) -> Self::Output {
		self + rhs.sign_extended::<i32>()
	}
}

impl<T> ops::AddAssign<T> for Pos
where
	Pos: ops::Add<T, Output = Self>,
{
	fn add_assign(&mut self, rhs: T) {
		*self = *self + rhs;
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

impl serde::Serialize for Pos {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		format_args!("{:#x}", self).serialize(serializer)
	}
}

impl<'de> serde::Deserialize<'de> for Pos {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		//deserializer.deserialize_any(PosVisitor)
		deserializer.deserialize_u32(PosVisitor)
	}
}

/// Visitor for deserializing a `Pos`.
struct PosVisitor;

impl<'de> serde::de::Visitor<'de> for PosVisitor {
	type Value = Pos;

	fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		formatter.write_str("a string-encoded hex value or `u32`")
	}

	fn visit_u64<E>(self, pos: u64) -> Result<Self::Value, E>
	where
		E: serde::de::Error,
	{
		#[allow(clippy::map_err_ignore)] // It's clearer to provide a string than the error from `try_from`
		let pos = u32::try_from(pos).map_err(|_| E::custom("Position must fit within a `u32`"))?;

		Ok(Pos(pos))
	}

	fn visit_i64<E>(self, pos: i64) -> Result<Self::Value, E>
	where
		E: serde::de::Error,
	{
		#[allow(clippy::map_err_ignore)] // It's clearer to provide a string than the error from `try_from`
		let pos = u32::try_from(pos).map_err(|_| E::custom("Position must fit within a `u32`"))?;

		Ok(Pos(pos))
	}

	fn visit_str<E>(self, pos: &str) -> Result<Self::Value, E>
	where
		E: serde::de::Error,
	{
		// If it doesn't begin with `0x`, error
		if !pos.starts_with("0x") {
			return Err(E::custom("String-encoded hex values must start with `0x`"));
		}

		u32::from_str_radix(pos.trim_start_matches("0x"), 16).map(Pos).map_err(E::custom)
	}
}
