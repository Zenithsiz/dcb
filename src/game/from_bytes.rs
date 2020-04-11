//! Converting from bytes to a game structure

// Traits
//--------------------------------------------------------------------------------------------------
	/// Represents a type that can be constructed from bytes
	/// from the game file.
	/// 
	/// # Details
	/// Types that implement this trait must be able to accept
	/// a buffer of the size `<Self as Bytes>::BUF_BYTE_SIZE`
	/// and construct themselves from data.
	pub trait FromBytes
	where
		Self: Sized
	{
		/// The error type used for the operation
		type Error;
		
		/// Reads `bytes` and returns a result with `Self`
		fn from_bytes(bytes: &[u8]) -> Result<Self, Self::Error>;
	}
//--------------------------------------------------------------------------------------------------
