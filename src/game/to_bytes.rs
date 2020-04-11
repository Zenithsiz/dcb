//! Converting from a game structure to bytes

// Traits
//--------------------------------------------------------------------------------------------------
	/// Represents a type that can be converted into bytes
	/// 
	/// # Details
	/// Types that implement this trait must be able to accept
	/// a buffer of the size `<Self as Bytes>::BUF_BYTE_SIZE`
	/// and fill the buffer with data from themselves.
	pub trait ToBytes
	where
		Self: Sized
	{
		/// The error type used for the operation
		type Error;
		
		/// Writes bytes into `bytes` from self
		fn to_bytes(&self, bytes: &mut [u8]) -> Result<(), Self::Error>;
	}
//--------------------------------------------------------------------------------------------------
