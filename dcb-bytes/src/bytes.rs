//! `Bytes` trait.

// Imports
use std::error::Error;

/// Conversion from and to bytes
pub trait Bytes
where
	Self: Sized,
{
	/// The type of array required by this structure
	type ByteArray: ByteArray;

	/// The error type used for the operation
	type FromError: Error;

	/// The error type used for the operation
	type ToError: Error;

	/// Constructs this structure from `bytes`
	fn from_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::FromError>;

	/// Writes this structure to `bytes`
	fn to_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::ToError>;

	/// Creates bytes and writes this structure to them
	fn bytes(&self) -> Result<Self::ByteArray, Self::ToError> {
		let mut bytes = Self::ByteArray::zeros();
		self.to_bytes(&mut bytes)?;
		Ok(bytes)
	}
}

/// A trait for restricting `Bytes::ByteArray`
pub trait ByteArray {
	/// Array size
	const SIZE: usize;

	/// Creates a new array filled with `0`s
	fn zeros() -> Self;
}

impl<const N: usize> ByteArray for [u8; N] {
	const SIZE: usize = N;

	fn zeros() -> Self {
		[0; N]
	}
}

impl ByteArray for u8 {
	const SIZE: usize = 1;

	fn zeros() -> Self {
		0
	}
}
