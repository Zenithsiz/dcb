//! String slice

// Imports
use super::Alphabet;
use std::{fmt, marker::PhantomData};

/// A alphabetic specific string slice
#[derive(PartialEq, Eq, PartialOrd, Ord)]
#[derive(ref_cast::RefCast)]
#[repr(transparent)]
pub struct StrAlphabet<A: Alphabet>(PhantomData<A>, [u8]);

impl<A: Alphabet> StrAlphabet<A> {
	/// Returns the bytes from this string
	#[must_use]
	pub fn as_bytes(&self) -> &[u8] {
		&self.1
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
		// Try to get self as a string to debug it
		// TODO: Not allocate here
		let s = String::from_utf8_lossy(self.as_bytes());

		// Then trim any spaces we might have
		let s = s.trim();

		write!(f, "{s:?}")
	}
}

impl<A: Alphabet> fmt::Display for StrAlphabet<A> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		// Try to get self as a string to debug it
		// TODO: Not allocate here
		let s = String::from_utf8_lossy(self.as_bytes());

		// Then trim any spaces we might have
		let s = s.trim();

		write!(f, "{s}")
	}
}
