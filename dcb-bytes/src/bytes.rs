//! `Bytes` trait.

// Imports
use std::error::Error;

/// Conversion from and to bytes
pub trait Bytes
where
	Self: Sized,
{
	/// The type of array required by this structure
	type ByteArray: ?Sized + ByteArray;

	/// The error type used for the operation
	type FromError: Error;

	/// The error type used for the operation
	type ToError: Error;

	/// Constructs this structure from `bytes`
	fn from_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::FromError>;

	/// Writes this structure to `bytes`
	fn to_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::ToError>;
}

/// A trait for restricting `Bytes::ByteArray`
pub trait ByteArray {
	/// Array size
	const SIZE: usize;
}

impl<const N: usize> ByteArray for [u8; N] {
	const SIZE: usize = N;
}

impl ByteArray for u8 {
	const SIZE: usize = 1;
}
