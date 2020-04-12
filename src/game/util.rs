//! Utility macros and functions
//! 
//! This modules is used for miscellaneous macros, functions that have
//! not been moved to a more permanent location.
//! 
//! All items in this module will eventually be depracated and moved
//! somewhere else, but this change might take some time.

// Macros


// Macros
//--------------------------------------------------------------------------------------------------
	/// Automatically generates `FromBytes` and `ToBytes` implementations for enum types
	macro_rules! generate_from_to_bytes
	{
		($name:ty, $bytes:expr, $err: tt, [ $($variant:ident => $value:expr,)* ]) =>
		{
			// Bytes
			impl $crate::game::Bytes for $name
			{
				const BUF_BYTE_SIZE : usize = $bytes;
			}
			
			// From bytes
			impl $crate::game::FromBytes for $name
			{
				type Error = $err;
				
				fn from_bytes(bytes: &[u8]) -> Result<Self, Self::Error>
				{
					match bytes[0] {
						$( $value => Ok( <$name>::$variant ), )*
						
						_ => { return Err( $err{ byte: bytes[0] } ); }
					}
				}
			}
			
			// To bytes
			impl $crate::game::ToBytes for $name
			{
				type Error = !;
				
				fn to_bytes(&self, bytes: &mut [u8]) -> Result<(), Self::Error>
				{
					bytes[0] = match self
					{
						$( <$name>::$variant => $value, )*
					};
					
					Ok(())
				}
			}
		}
	}
//--------------------------------------------------------------------------------------------------

// Types
//--------------------------------------------------------------------------------------------------
	/// Error type for `read_null_terminated_string`
	#[derive(Debug, derive_more::Display)]
	pub enum ReadNullTerminatedStringError
	{
		/// No null was found on a string
		#[display(fmt = "No null was found on a null terminated string")]
		NoNull,
		
		/// A string could not be converted to utf8
		#[display(fmt = "Could not convert the string to utf8")]
		Utf8( std::str::Utf8Error ),
	}
	
	/// Error type for `write_null_terminated_string`
	#[derive(Debug, derive_more::Display)]
	#[display(fmt = "The string was too long to write to the buffer ({0} / {1})", string_size, buf_size)]
	pub struct WriteNullTerminatedStringError
	{
		/// The string size
		string_size: usize,
		
		/// The buffer size
		buf_size: usize,
	}
//--------------------------------------------------------------------------------------------------

// Impl
//--------------------------------------------------------------------------------------------------
	
//--------------------------------------------------------------------------------------------------

// Functions
//--------------------------------------------------------------------------------------------------
	/// Reads a string from a buffer, stopping at the first null character found
	/// 
	/// # Errors
	/// - `NoNull`: If no null character was found until the end of the buffer.
	/// - `Utf8`: If the buffer was not valid utf8.
	pub fn read_null_terminated_string(mut buf: &[u8]) -> Result<&str, ReadNullTerminatedStringError>
	{
		// Search for the first occurence of null and reduce the buffer to before it.
		// If not found, then the string was not null terminated, so return Err
		if let Some(first_null) = buf.iter().position(|&b| b == 0) { buf = &buf[0..first_null]; }
		else { return Err( ReadNullTerminatedStringError::NoNull ); }
		
		// Else try to conver the buffer into a utf8 str.
		Ok( std::str::from_utf8( buf ).map_err(ReadNullTerminatedStringError::Utf8)? )
	}
	
	/// Writes a string to a buffer with a null terminator and returns it
	/// 
	/// # Details
	/// Will reserve the last byte after the string for a null, none of
	/// the bytes after it will be touched.
	/// 
	/// # Errors
	/// - `TooLong`: If the string is too long for the buffer
	pub fn write_null_terminated_string<'a>(string: &'_ str, buf: &'a mut [u8]) -> Result<&'a mut [u8], WriteNullTerminatedStringError>
	{
		// Check if the string is too big
		// Note: This also catches the case where they're both 0
		if string.len() >= buf.len() { return Err( WriteNullTerminatedStringError{ string_size: string.len(), buf_size: buf.len() } ); }
		
		// Else copy everything over
		buf[ 0..string.len() ].copy_from_slice( string.as_bytes() );
		
		// Set the last byte of the string as null
		buf[ string.len() ] = 0;
		
		// And return the buffer
		Ok( buf )
	}
	
	/// Returns an ordinal string from a u64
	#[must_use]
	pub fn as_ordinal(num: u64) -> String
	{
		format!("{0}{1}", num, match num % 10 {
			1 => "st",
			2 => "nd",
			3 => "rd",
			_ => "th",
		})
	}
//--------------------------------------------------------------------------------------------------
