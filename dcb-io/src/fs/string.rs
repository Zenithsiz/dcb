//! Filesystem strings

/// Modules
pub mod error;

// Exports
pub use error::InvalidCharError;

// Imports
use std::{fmt, marker::PhantomData};

/// An alphabet
pub trait Alphabet {
	/// The alphabet
	fn alphabet() -> &'static [u8];
}

/// A alphabetic specific string
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct StrArrAlphabet<A: Alphabet, const N: usize>([u8; N], PhantomData<A>);

impl<A: Alphabet, const N: usize> StrArrAlphabet<A, N> {
	/// Parses a string from bytes
	pub fn from_bytes(bytes: &[u8; N]) -> Result<Self, InvalidCharError> {
		// If any are invalid, return Err
		let alphabet = A::alphabet();
		for (pos, &byte) in bytes.iter().enumerate() {
			// If we found a space, as long as everything after this position is also a space, it's a valid string
			if byte == b' ' {
				match bytes[pos..].iter().all(|&b| b == b' ') {
					true => break,
					false => return Err(InvalidCharError { byte, pos }),
				};
			}

			if !alphabet.contains(&byte) {
				return Err(InvalidCharError { byte, pos });
			}
		}

		Ok(Self(*bytes, PhantomData))
	}

	/// Returns the bytes from this string
	#[must_use]
	pub fn as_bytes(&self) -> &[u8; N] {
		&self.0
	}
}

impl<A: Alphabet, const N: usize> fmt::Debug for StrArrAlphabet<A, N> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		// Try to get self as a string to debug it
		// TODO: Not allocate here
		let s = String::from_utf8_lossy(self.as_bytes());

		// Then trim any spaces we might have
		let s = s.trim();

		write!(f, "{s:?}")
	}
}

impl<A: Alphabet, const N: usize> fmt::Display for StrArrAlphabet<A, N> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		// Try to get self as a string to debug it
		// TODO: Not allocate here
		let s = String::from_utf8_lossy(self.as_bytes());

		// Then trim any spaces we might have
		let s = s.trim();

		write!(f, "{s}")
	}
}


/// A-type alphabet
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct AlphabetA;

impl Alphabet for AlphabetA {
	fn alphabet() -> &'static [u8] {
		&[
			b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H', b'I', b'J', b'K', b'L', b'M', b'N', b'O', b'P', b'Q', b'R', b'S', b'T', b'U', b'V', b'W',
			b'X', b'Y', b'Z', b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'_', b' ', b'!', b'"', b'%', b'&', b'\'', b'(', b')',
			b'*', b'+', b',', b'-', b'.', b'/', b':', b';', b'<', b'=', b'>', b'?',
		]
	}
}

/// D-type alphabet
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct AlphabetD;

impl Alphabet for AlphabetD {
	fn alphabet() -> &'static [u8] {
		&[
			b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H', b'I', b'J', b'K', b'L', b'M', b'N', b'O', b'P', b'Q', b'R', b'S', b'T', b'U', b'V', b'W',
			b'X', b'Y', b'Z', b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'_',
		]
	}
}

/// A-type string
pub type StrArrA<const N: usize> = StrArrAlphabet<AlphabetA, N>;

/// D-type string
pub type StrArrD<const N: usize> = StrArrAlphabet<AlphabetD, N>;
