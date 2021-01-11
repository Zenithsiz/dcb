//! Instruction position

// Imports
use crate::Exe;
use int_conv::{SignExtended, Signed};
use std::{convert::TryFrom, fmt, ops};

/// An instruction position
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, Debug)]
#[derive(derive_more::Display)]
#[derive(ref_cast::RefCast)]
#[display(fmt = "{_0:#x}")]
#[repr(transparent)]
pub struct Pos(pub u32);

impl Pos {
	/// Adds a `u32` offset to this position
	// TODO: Remove once we can `impl const Add`
	#[must_use]
	pub(crate) const fn add_offset_u32(self, offset: u32) -> Self {
		Self(self.0 + offset)
	}

	/// Returns the memory position of this position
	#[must_use]
	pub fn as_mem_idx(self) -> usize {
		usize::try_from(self - Exe::MEM_START_ADDRESS).expect("Failed to compute index")
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
		Self(self.0 + rhs)
	}
}

// `Pos + i32 = Pos`
impl ops::Add<i32> for Pos {
	type Output = Self;

	fn add(self, rhs: i32) -> Self::Output {
		Self((self.0.as_signed() + rhs).as_unsigned())
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
		Self(self.0 - rhs)
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

impl serde::Serialize for Pos {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		format_args!("{}", self).serialize(serializer)
	}
}

impl<'de> serde::Deserialize<'de> for Pos {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		deserializer.deserialize_u32(PosVisitor)
	}
}

/// Visitor for deserializing a `Pos`.
struct PosVisitor;

#[allow(clippy::map_err_ignore)] // It's clearer to provide a string than the error from `try_from`
impl<'de> serde::de::Visitor<'de> for PosVisitor {
	type Value = Pos;

	fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		formatter.write_str("a string-encoded hex value or `u32`")
	}

	fn visit_u64<E>(self, pos: u64) -> Result<Self::Value, E>
	where
		E: serde::de::Error,
	{
		let pos = u32::try_from(pos).map_err(|_| E::custom("Position must fit within a `u32`"))?;
		Ok(Pos(pos))
	}

	fn visit_i64<E>(self, pos: i64) -> Result<Self::Value, E>
	where
		E: serde::de::Error,
	{
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
