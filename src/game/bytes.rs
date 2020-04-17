//! Interface for converting various structures to and from bytes

/// Convertions to and from bytes for the game file
pub trait Bytes
where
	Self: Sized
{
	/// The size of the structure in bytes
	const BUF_BYTE_SIZE: usize;
	
	/// The error type used for the operation
	type FromError;
		
	/// Reads `bytes` and returns a result with `Self`
	fn from_bytes(bytes: &[u8]) -> Result<Self, Self::FromError>;
	
	/// The error type used for the operation
	type ToError;
	
	/// Writes bytes into `bytes` from self
	fn to_bytes(&self, bytes: &mut [u8]) -> Result<(), Self::ToError>;
}
