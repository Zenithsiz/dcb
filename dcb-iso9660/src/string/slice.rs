//! String slice

// Imports
use super::Alphabet;
use core::str::lossy::Utf8Lossy;
use ref_cast::RefCast;
use std::{fmt, marker::PhantomData};

/// An alphabetic string slice
// TODO: Not expose `ref_cast` to the outside, as it breaks
//       this string's encapsulation.
#[derive(PartialEq, Eq, PartialOrd, Ord)]
#[derive(ref_cast::RefCast)]
#[repr(transparent)]
pub struct StrAlphabet<A: Alphabet> {
	/// Phantom
	phantom: PhantomData<A>,

	/// Bytes
	bytes: [u8],
}

impl<A: Alphabet> StrAlphabet<A> {
	/// Returns the bytes from this string
	#[must_use]
	pub fn as_bytes(&self) -> &[u8] {
		&self.bytes
	}

	/// Parses a string from bytes
	pub fn from_bytes(bytes: &[u8]) -> Result<&Self, A::Error> {
		A::validate(bytes).map(|bytes| Self::ref_cast(bytes))
	}

	/// Returns this string as a lossy `str`
	#[must_use]
	pub fn as_lossy_str(&self) -> &Utf8Lossy {
		Utf8Lossy::from_bytes(self.as_bytes())
	}

	/// Returns the length of this string
	#[must_use]
	pub fn len(&self) -> usize {
		self.as_bytes().len()
	}

	/// Returns if this string is empty
	#[must_use]
	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}
}

impl<A: Alphabet> fmt::Debug for StrAlphabet<A> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{:?}", self.as_lossy_str())
	}
}

impl<A: Alphabet> fmt::Display for StrAlphabet<A> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.as_lossy_str())
	}
}
