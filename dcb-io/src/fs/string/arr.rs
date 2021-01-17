//! String array

// Imports
use super::{Alphabet, StrAlphabet};
use std::{fmt, marker::PhantomData, ops::Deref};


/// A alphabetic specific string array
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct StrArrAlphabet<A: Alphabet, const N: usize>(PhantomData<A>, [u8; N]);

impl<A: Alphabet, const N: usize> StrArrAlphabet<A, N> {
	/// Parses a string from bytes
	pub fn from_bytes(bytes: &[u8; N]) -> Result<Self, A::Error> {
		A::validate(bytes).map(|()| Self(PhantomData, *bytes))
	}
}

impl<A: Alphabet, const N: usize> Deref for StrArrAlphabet<A, N> {
	type Target = StrAlphabet<A>;

	fn deref(&self) -> &Self::Target {
		ref_cast::RefCast::ref_cast(self.1.as_slice())
	}
}

impl<A: Alphabet, const N: usize> fmt::Debug for StrArrAlphabet<A, N> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{:?}", &*self)
	}
}

impl<A: Alphabet, const N: usize> fmt::Display for StrArrAlphabet<A, N> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", &*self)
	}
}
