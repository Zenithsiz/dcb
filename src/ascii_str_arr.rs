//! Ascii string backed by an array

// Modules
pub mod error;
pub mod slice;
mod visitor;

// Exports
pub use error::{FromBytesError, NotAsciiError, TooLongError};
pub use slice::SliceIndex;

// Imports
use ascii::{AsciiChar, AsciiStr};
use std::{cmp::Ordering, convert::TryFrom, fmt, hash::Hash};
use visitor::DeserializerVisitor;

/// An ascii string backed by an array
#[derive(Eq, Clone, Copy)]
pub struct AsciiStrArr<const N: usize> {
	/// Characters
	chars: [AsciiChar; N],

	/// Size
	// Invariant `self.len <= N`
	len: usize,
}

// Length interface
impl<const N: usize> AsciiStrArr<N> {
	/// Returns the length of this string
	#[must_use]
	#[contracts::debug_ensures(self.len <= N)]
	pub fn len(&self) -> usize {
		self.len
	}

	/// Returns if this string is empty
	#[must_use]
	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}
}

// Conversions into other strings
impl<const N: usize> AsciiStrArr<N> {
	/// Converts this string to a `&AsciiStr`
	#[must_use]
	pub fn as_ascii(&self) -> &AsciiStr {
		// SAFETY: `self.len <= N`
		let chars = unsafe { self.chars.get_unchecked(..self.len()) };
		chars.into()
	}

	/// Converts this string to a `&mut AsciiStr`
	#[must_use]
	pub fn as_ascii_mut(&mut self) -> &mut AsciiStr {
		// SAFETY: `self.len <= N`
		let len = self.len();
		let chars = unsafe { self.chars.get_unchecked_mut(..len) };
		chars.into()
	}

	/// Converts this string to a `&[u8]`
	#[must_use]
	pub fn as_bytes(&self) -> &[u8] {
		self.as_ascii().as_bytes()
	}

	/// Converts this string to a `&str`
	#[must_use]
	pub fn as_str(&self) -> &str {
		self.as_ascii().as_str()
	}
}

/// Conversions from other strings
impl<const N: usize> AsciiStrArr<N> {
	/// Creates a string from a `&AsciiStr`
	pub fn from_ascii(ascii: &AsciiStr) -> Result<Self, TooLongError<N>> {
		// If we can't fit it, return Err
		let len = ascii.len();
		if len > N {
			return Err(TooLongError::<N>);
		}

		// Else copy everything over and return ourselves
		let mut chars = [AsciiChar::Null; N];
		chars[0..len].copy_from_slice(ascii.as_ref());
		Ok(Self { chars, len })
	}

	/// Creates a string from a `&[u8]`
	pub fn from_bytes(bytes: &[u8]) -> Result<Self, FromBytesError<N>> {
		// Get the bytes as ascii first
		let ascii = AsciiStr::from_ascii(bytes).map_err(FromBytesError::NotAscii)?;

		// Then try to convert them
		Self::from_ascii(ascii).map_err(FromBytesError::TooLong)
	}

	// Note: No `from_str`, implemented using `FromStr`
}

// Slicing
impl<const N: usize> AsciiStrArr<N> {
	/// Slices this string, if in bounds
	#[must_use]
	pub fn get<I: SliceIndex>(&self, idx: I) -> Option<&I::Output> {
		idx.get(self)
	}

	/// Slices this string mutably, if in bounds
	#[must_use]
	pub fn get_mut<I: SliceIndex>(&mut self, idx: I) -> Option<&mut I::Output> {
		idx.get_mut(self)
	}

	/// Slices the string without checking bounds
	///
	/// # Safety
	/// Calling this method with an out-of-bounds index is undefined-behavior even if the resulting
	/// reference is not used
	#[must_use]
	pub unsafe fn get_unchecked<I: SliceIndex>(&self, idx: I) -> &I::Output {
		idx.get_unchecked(self)
	}

	/// Slices the string mutably without checking bounds
	///
	/// # Safety
	/// Calling this method with an out-of-bounds index is undefined-behavior even if the resulting
	/// reference is not used
	#[must_use]
	pub unsafe fn get_unchecked_mut<I: SliceIndex>(&mut self, idx: I) -> &mut I::Output {
		idx.get_unchecked_mut(self)
	}
}

impl<const N: usize> AsRef<AsciiStr> for AsciiStrArr<N> {
	fn as_ref(&self) -> &AsciiStr {
		self.as_ascii()
	}
}

impl<const N: usize> AsMut<AsciiStr> for AsciiStrArr<N> {
	fn as_mut(&mut self) -> &mut AsciiStr {
		self.as_ascii_mut()
	}
}

impl<const N: usize> AsRef<[u8]> for AsciiStrArr<N> {
	fn as_ref(&self) -> &[u8] {
		self.as_bytes()
	}
}

impl<const N: usize> AsRef<str> for AsciiStrArr<N> {
	fn as_ref(&self) -> &str {
		self.as_str()
	}
}

impl<const N: usize> PartialEq for AsciiStrArr<N> {
	fn eq(&self, other: &Self) -> bool {
		AsciiStr::eq(self.as_ascii(), other.as_ascii())
	}
}

impl<const N: usize> PartialOrd for AsciiStrArr<N> {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		AsciiStr::partial_cmp(self.as_ascii(), other.as_ascii())
	}
}

impl<const N: usize> Ord for AsciiStrArr<N> {
	fn cmp(&self, other: &Self) -> Ordering {
		AsciiStr::cmp(self.as_ascii(), other.as_ascii())
	}
}

impl<const N: usize> Hash for AsciiStrArr<N> {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		AsciiStr::hash(self.as_ascii(), state)
	}
}

impl<const N: usize> fmt::Debug for AsciiStrArr<N> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		AsciiStr::fmt(self.as_ascii(), f)
	}
}

impl<const N: usize> fmt::Display for AsciiStrArr<N> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		AsciiStr::fmt(self.as_ascii(), f)
	}
}

impl<'de, const N: usize> serde::Deserialize<'de> for AsciiStrArr<N> {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		match deserializer.deserialize_str(DeserializerVisitor) {
			Ok(string) => Ok(string),
			Err(err) => Err(err),
		}
	}
}

impl<const N: usize> serde::Serialize for AsciiStrArr<N> {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		serializer.serialize_str(self.as_ascii().as_str())
	}
}

// TODO: Generalize this to `impl<const N: usize, const M: usize> From<&[AsciiChar; M]> for AsciiStrArr<N> where M <= N`
impl<const N: usize> From<&[AsciiChar; N]> for AsciiStrArr<N> {
	fn from(src: &[AsciiChar; N]) -> Self {
		Self::from(*src)
	}
}

// TODO: Generalize this to `impl<const N: usize, const M: usize> From<[AsciiChar; M]> for AsciiStrArr<N> where M <= N`
impl<const N: usize> From<[AsciiChar; N]> for AsciiStrArr<N> {
	fn from(chars: [AsciiChar; N]) -> Self {
		Self { chars, len: N }
	}
}

// TODO: Generalize this to `impl<const N: usize, const M: usize> TryFrom<&[u8; M]> for AsciiStrArr<N> where M <= N`
impl<const N: usize> TryFrom<&[u8; N]> for AsciiStrArr<N> {
	type Error = NotAsciiError;

	fn try_from(byte_str: &[u8; N]) -> Result<Self, Self::Error> {
		let mut ascii_str = [AsciiChar::Null; N];
		for (pos, &byte) in byte_str.iter().enumerate() {
			ascii_str[pos] = AsciiChar::from_ascii(byte).map_err(|_| NotAsciiError { pos })?;
		}

		Ok(Self::from(ascii_str))
	}
}

impl<const N: usize> TryFrom<&AsciiStr> for AsciiStrArr<N> {
	type Error = TooLongError<N>;

	fn try_from(ascii: &AsciiStr) -> Result<Self, Self::Error> {
		Self::from_ascii(ascii)
	}
}

impl<const N: usize> TryFrom<&[u8]> for AsciiStrArr<N> {
	type Error = FromBytesError<N>;

	fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
		Self::from_bytes(bytes)
	}
}

impl<const N: usize> TryFrom<&str> for AsciiStrArr<N> {
	type Error = FromBytesError<N>;

	fn try_from(string: &str) -> Result<Self, Self::Error> {
		let ascii_str = AsciiStr::from_ascii(string).map_err(FromBytesError::NotAscii)?;

		Self::try_from(ascii_str).map_err(FromBytesError::TooLong)
	}
}

impl<const N: usize> std::str::FromStr for AsciiStrArr<N> {
	type Err = FromBytesError<N>;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		// Simply delegate to [`Self::from_bytes`]
		Self::from_bytes(s.as_bytes())
	}
}
