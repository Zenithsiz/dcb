//! Ascii string backed by an array

// Modules
pub mod error;
pub mod slice;
#[cfg(test)]
mod test;
mod visitor;

// Exports
pub use error::{FromBytesError, NotAsciiError, TooLongError};
pub use slice::SliceIndex;

// Imports
use ascii::{AsciiChar, AsciiStr};
use std::{cmp::Ordering, convert::TryFrom, fmt, hash::Hash, mem::MaybeUninit};
use visitor::DeserializerVisitor;

/// An ascii string backed by an array
#[derive(Clone, Copy)]
pub struct AsciiStrArr<const N: usize> {
	/// Characters
	// Invariant: First `len` elements are initialized.
	chars: [MaybeUninit<AsciiChar>; N],

	/// Size
	// Invariant `self.len <= N`
	// Note: On all methods except `Self::len()`, one should call `self.len()` instead of using `self.len`.
	len: usize,
}

// Constructors
impl<const N: usize> AsciiStrArr<N> {
	/// Creates a new empty string
	#[must_use]
	pub fn new() -> Self {
		Self {
			chars: MaybeUninit::uninit_array(),
			len:   0,
		}
	}
}

/// String lengths
impl<const N: usize> AsciiStrArr<N> {
	/// The capacity of the string
	pub const CAPACITY: usize = N;

	/// Returns the length of this string
	#[must_use]
	#[contracts::debug_ensures(self.len <= N)]
	pub fn len(&self) -> usize {
		self.len
	}

	/// Returns the capacity of the string, `N`
	#[must_use]
	pub const fn capacity() -> usize {
		Self::CAPACITY
	}

	/// Sets this string's length
	///
	/// # Safety
	/// - All elements `0..new_len` must be initialized.
	/// - `new_len` must be less or equal to `N`.
	#[contracts::debug_requires(new_len <= N)]
	pub unsafe fn set_len(&mut self, new_len: usize) {
		self.len = new_len;
	}

	/// Returns if this string is empty
	#[must_use]
	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}
}

/// Conversions to other string types
impl<const N: usize> AsciiStrArr<N> {
	/// Converts this string to a `&AsciiStr`
	#[must_use]
	pub fn as_ascii(&self) -> &AsciiStr {
		// Get all the initialized elements
		// SAFETY: `self.len <= N`
		let chars = unsafe { self.chars.get_unchecked(..self.len()) };

		// Then get a reference to them
		// SAFETY: The first `self.len` elements are initialized
		let chars = unsafe { MaybeUninit::slice_assume_init_ref(chars) };

		<&AsciiStr>::from(chars)
	}

	/// Converts this string to a `&mut AsciiStr`
	#[must_use]
	pub fn as_ascii_mut(&mut self) -> &mut AsciiStr {
		// Get all the initialized elements
		// SAFETY: `self.len <= N`
		let len = self.len();
		let chars = unsafe { self.chars.get_unchecked_mut(..len) };

		// Then get a mutable reference to them
		// SAFETY: The first `self.len` elements are initialized
		let chars = unsafe { MaybeUninit::slice_assume_init_mut(chars) };

		<&mut AsciiStr>::from(chars)
	}

	/// Converts this string to a `&[AsciiChar]`
	#[must_use]
	pub fn as_ascii_slice(&self) -> &[AsciiChar] {
		self.as_ascii().as_slice()
	}

	/// Converts this string to a `&mut [AsciiChar]`
	#[must_use]
	pub fn as_ascii_slice_mut(&mut self) -> &mut [AsciiChar] {
		self.as_ascii_mut().as_mut_slice()
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

	/// Returns a pointer to the initialized elements
	///
	/// # Safety
	/// The returned pointer is only valid for `self.len()` elements
	/// The returned pointer is invalidated if `self` is moved.
	#[must_use]
	pub fn as_ptr(&self) -> *const AsciiChar {
		self.as_ascii().as_ptr()
	}

	/// Returns a mutable pointer to the initialized elements
	///
	/// # Safety
	/// The returned pointer is only valid for `self.len()` elements
	/// The returned pointer is invalidated if `self` is moved.
	#[must_use]
	pub fn as_ptr_mut(&mut self) -> *mut AsciiChar {
		self.as_ascii_mut().as_mut_ptr()
	}

	/// Exposes the underlying buffer this string contains
	///
	/// # Safety
	/// All elements `0..self.len()` must remain initialized.
	pub unsafe fn buffer_mut(&mut self) -> &mut [MaybeUninit<AsciiChar>; N] {
		&mut self.chars
	}
}

/// Conversions from other strings
impl<const N: usize> AsciiStrArr<N> {
	/// Creates a string from anything that coerces to `&[AsciiChar]`, including `AsciiStr`
	pub fn from_ascii<S: ?Sized + AsRef<[AsciiChar]>>(ascii: &S) -> Result<Self, TooLongError<N>> {
		let ascii = ascii.as_ref();

		// If it has too many elements, return Err
		let len = ascii.len();
		if len > N {
			return Err(TooLongError::<N>);
		}

		// Else create an uninitialized array and copy over the initialized characters
		let mut chars = MaybeUninit::uninit_array();
		for (idx, c) in chars.iter_mut().take(len).enumerate() {
			// Write each character
			// SAFETY: `idx` is within bounds (`0..len`)
			// Note: `MaybeUnit::drop` is a no-op, so we can use normal assignment.
			*c = MaybeUninit::new(*unsafe { ascii.get_unchecked(idx) });
		}

		Ok(Self { chars, len })
	}

	/// Creates a string from a `&[u8]`
	pub fn from_bytes<B: ?Sized + AsRef<[u8]>>(bytes: &B) -> Result<Self, FromBytesError<N>> {
		// Get the bytes as ascii first
		let ascii = AsciiStr::from_ascii(bytes).map_err(FromBytesError::NotAscii)?;

		// Then try to convert them
		Self::from_ascii(ascii).map_err(FromBytesError::TooLong)
	}

	// Note: No `from_str`, implemented using `FromStr`
}

/// Slicing
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
		// SAFETY: Index is guaranteed to be in bounds by the caller
		unsafe { idx.get_unchecked(self) }
	}

	/// Slices the string mutably without checking bounds
	///
	/// # Safety
	/// Calling this method with an out-of-bounds index is undefined-behavior even if the resulting
	/// reference is not used
	#[must_use]
	pub unsafe fn get_unchecked_mut<I: SliceIndex>(&mut self, idx: I) -> &mut I::Output {
		// SAFETY: Index is guaranteed to be in bounds by the caller
		unsafe { idx.get_unchecked_mut(self) }
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

impl<const N: usize> AsRef<[AsciiChar]> for AsciiStrArr<N> {
	fn as_ref(&self) -> &[AsciiChar] {
		self.as_ascii_slice()
	}
}

impl<const N: usize> AsMut<[AsciiChar]> for AsciiStrArr<N> {
	fn as_mut(&mut self) -> &mut [AsciiChar] {
		self.as_ascii_slice_mut()
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

// Note: No `AsMut<[u8]>` nor `AsMut<str>`, as that'd allow for modification
//       outside of ascii.

impl<const N: usize> PartialEq for AsciiStrArr<N> {
	fn eq(&self, other: &Self) -> bool {
		AsciiStr::eq(self.as_ascii(), other.as_ascii())
	}
}

impl<const N: usize> Eq for AsciiStrArr<N> {}

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

impl<const N: usize> Default for AsciiStrArr<N> {
	fn default() -> Self {
		Self::new()
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
		let chars = chars.map(MaybeUninit::new);
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
