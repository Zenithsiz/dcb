//! String array

// Imports
use super::{Alphabet, StrAlphabet};
use std::{fmt, marker::PhantomData, ops::Deref};

/// An alphabetic string array
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct StrArrAlphabet<A: Alphabet, const N: usize> {
	/// Phantom
	phantom: PhantomData<A>,

	/// Bytes
	bytes: [u8; N],

	/// Length
	len: usize,
}

impl<A: Alphabet, const N: usize> StrArrAlphabet<A, N> {
	/// Parses a string from bytes
	pub fn from_bytes(bytes: &[u8; N]) -> Result<Self, A::Error> {
		A::validate(bytes).map(|valid_bytes| Self {
			phantom: PhantomData,
			bytes:   *bytes,
			len:     valid_bytes.len(),
		})
	}
}

impl<A: Alphabet, const N: usize> Deref for StrArrAlphabet<A, N> {
	type Target = StrAlphabet<A>;

	fn deref(&self) -> &Self::Target {
		ref_cast::RefCast::ref_cast(&self.bytes.as_slice()[..self.len])
	}
}

impl<A: Alphabet, const N: usize> fmt::Debug for StrArrAlphabet<A, N> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let s: &StrAlphabet<A> = self;
		write!(f, "{:?}", s)
	}
}

impl<A: Alphabet, const N: usize> fmt::Display for StrArrAlphabet<A, N> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let s: &StrAlphabet<A> = self;
		write!(f, "{}", s)
	}
}
