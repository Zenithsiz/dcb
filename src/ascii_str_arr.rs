//! Ascii string backed by an array

// Modules
pub mod error;
pub mod slice;
mod visitor;

// Exports
pub use error::{FromByteStringError, NotAsciiError, TooLongError};

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
	///
	/// Invariant: Must be `< N`
	len: usize,
}

impl<const N: usize> AsciiStrArr<N> {
	/// Returns the length of this string
	#[must_use]
	pub const fn len(&self) -> usize {
		self.len
	}

	/// Returns if this string is empty
	#[must_use]
	pub const fn is_empty(&self) -> bool {
		self.len() == 0
	}

	/// Converts this string to a `&[AsciiStr]`
	#[must_use]
	pub fn as_ascii_str(&self) -> &AsciiStr {
		// Note: Cannot panic due to our invariant
		<&AsciiStr>::from(&self.chars[..self.len])
	}

	/// Converts this string to a `&mut [AsciiStr]`
	#[must_use]
	pub fn as_ascii_str_mut(&mut self) -> &mut AsciiStr {
		// Note: Cannot panic due to our invariant
		<&mut AsciiStr>::from(&mut self.chars[..self.len])
	}

	/// Converts this string to a `&[u8]`
	#[must_use]
	pub fn as_bytes(&self) -> &[u8] {
		self.as_ascii_str().as_bytes()
	}

	/// Converts this string to a `&str`
	#[must_use]
	pub fn as_str(&self) -> &str {
		self.as_ascii_str().as_str()
	}
}

impl<const N: usize> AsRef<AsciiStr> for AsciiStrArr<N> {
	fn as_ref(&self) -> &AsciiStr {
		self.as_ascii_str()
	}
}

impl<const N: usize> AsMut<AsciiStr> for AsciiStrArr<N> {
	fn as_mut(&mut self) -> &mut AsciiStr {
		self.as_ascii_str_mut()
	}
}

impl<const N: usize> PartialEq for AsciiStrArr<N> {
	fn eq(&self, other: &Self) -> bool {
		AsciiStr::eq(self.as_ascii_str(), other.as_ascii_str())
	}
}

impl<const N: usize> PartialOrd for AsciiStrArr<N> {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		AsciiStr::partial_cmp(self.as_ascii_str(), other.as_ascii_str())
	}
}

impl<const N: usize> Ord for AsciiStrArr<N> {
	fn cmp(&self, other: &Self) -> Ordering {
		AsciiStr::cmp(self.as_ascii_str(), other.as_ascii_str())
	}
}

impl<const N: usize> Hash for AsciiStrArr<N> {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		AsciiStr::hash(self.as_ascii_str(), state)
	}
}

impl<const N: usize> fmt::Debug for AsciiStrArr<N> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		AsciiStr::fmt(self.as_ascii_str(), f)
	}
}

impl<const N: usize> fmt::Display for AsciiStrArr<N> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		AsciiStr::fmt(self.as_ascii_str(), f)
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
		serializer.serialize_str(self.as_ascii_str().as_str())
	}
}

// TODO: Generalize this to `impl<const N: usize, const M: usize> From<&[AsciiChar; M]> for AsciiStrArr<N> where M <= N`
impl<const N: usize> From<&[AsciiChar; N]> for AsciiStrArr<N> {
	fn from(src: &[AsciiChar; N]) -> Self {
		let mut chars = [AsciiChar::Null; N];
		chars.copy_from_slice(src);
		Self { chars, len: N }
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

	fn try_from(ascii_str: &AsciiStr) -> Result<Self, Self::Error> {
		// Note: No space for null, this isn't null terminated
		let len = ascii_str.len();
		if len > N {
			return Err(TooLongError::<N>);
		}

		let mut chars = [AsciiChar::Null; N];
		chars[0..len].copy_from_slice(ascii_str.as_ref());
		Ok(Self { chars, len })
	}
}

impl<const N: usize> TryFrom<&[u8]> for AsciiStrArr<N> {
	type Error = FromByteStringError<N>;

	fn try_from(byte_str: &[u8]) -> Result<Self, Self::Error> {
		let ascii_str = AsciiStr::from_ascii(byte_str).map_err(FromByteStringError::NotAscii)?;

		Self::try_from(ascii_str).map_err(FromByteStringError::TooLong)
	}
}

impl<const N: usize> TryFrom<&str> for AsciiStrArr<N> {
	type Error = FromByteStringError<N>;

	fn try_from(string: &str) -> Result<Self, Self::Error> {
		let ascii_str = AsciiStr::from_ascii(string).map_err(FromByteStringError::NotAscii)?;

		Self::try_from(ascii_str).map_err(FromByteStringError::TooLong)
	}
}
