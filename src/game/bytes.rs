//! Interface for converting various structures to and from bytes

// Modules
pub mod validation;

// Exports
pub use validation::Validation;

// Std
use std::{error::Error, fmt::Debug};

/// Conversions to and from bytes for the game file
pub trait Bytes
where
	Self: Sized,
{
	/// The type of array required by this structure
	///
	/// *MUST* be a `[u8; N]` or `u8`
	type ByteArray: Sized;

	/// The error type used for the operation
	type FromError: Debug + Error;

	/// The error type used for the operation
	type ToError: Debug + Error;

	/// Constructs this structure from `bytes`
	fn from_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::FromError>;

	/// Writes this structure to `bytes`
	fn to_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::ToError>;
}
