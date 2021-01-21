//! String

// Imports
use super::{Alphabet, StrAlphabet};
use std::{fmt, marker::PhantomData, ops::Deref};

/// A alphabetic specific string
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct StringAlphabet<A: Alphabet>(PhantomData<A>, Vec<u8>);

impl<A: Alphabet> StringAlphabet<A> {
	/// Parses a string from bytes
	pub fn from_bytes(bytes: &[u8]) -> Result<Self, A::Error> {
		A::validate(bytes).map(|()| Self(PhantomData, bytes.to_vec()))
	}
}

impl<A: Alphabet> Deref for StringAlphabet<A> {
	type Target = StrAlphabet<A>;

	fn deref(&self) -> &Self::Target {
		ref_cast::RefCast::ref_cast(self.1.as_slice())
	}
}

impl<A: Alphabet> fmt::Debug for StringAlphabet<A> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let s: &StrAlphabet<A> = self;
		write!(f, "{:?}", s)
	}
}

impl<A: Alphabet> fmt::Display for StringAlphabet<A> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let s: &StrAlphabet<A> = self;
		write!(f, "{}", s)
	}
}
