//! Interface for converting various structures to and from bytes

/// Convertions to and from bytes for the game file
pub trait Bytes
where
	Self: Sized
{
	/// The type of array required by this structure
	type ByteArray: Sized;
	
	/// The error type used for the operation
	type FromError: std::fmt::Debug + std::error::Error;
		
	/// Reads `bytes` and returns a result with `Self`
	fn from_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::FromError>;
	
	/// The error type used for the operation
	type ToError: std::fmt::Debug + std::error::Error;
	
	/// Writes bytes into `bytes` from self
	fn to_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::ToError>;
}
