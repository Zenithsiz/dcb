//! Utility macros and functions
//! 
//! This modules is used for miscellaneous macros, functions that have
//! not been moved to a more permanent location.
//! 
//! All items in this module will eventually be depracated and moved
//! somewhere else, but this change might take some time.


pub macro array_split {
	(
		$arr:expr,
		$(
			$name:ident :
			
			$( [$arr_size:expr]    )?
			$(  $val_size:literal  )?
			
		),* $(,)?
	) => {{
		#![allow(clippy::used_underscore_binding)]
		#![allow(clippy::ptr_offset_with_cast   )]
		
		// Struct holding all fields
		struct Fields<'a, T> {
			$(
				$name:
				
				$( &'a [T; $arr_size], )?
				$( &'a T, #[cfg(os = "Os that does not exist")] __field: [u8; $val_size], )?
			)*
		}
		
		// Get everything from `array_refs`
		let (
			$(
				$name
			),*
		) = ::arrayref::array_refs!(
			$arr,
			$(
				$( $arr_size )?
				$( $val_size )?
			),*
		);
		
		// And return the fields
		Fields {
			$(
				$name
				$( : &( $name[$val_size - $val_size] ) )?
				,
			)*
		}
	}}
}

pub macro array_split_mut {
	(
		$arr:expr,
		$(
			$name:ident :
			
			$( [$arr_size:expr]    )?
			$(  $val_size:literal  )?
			
		),* $(,)?
	) => {{
		#![allow(clippy::used_underscore_binding)]
		#![allow(clippy::ptr_offset_with_cast   )]
		
		// Struct holding all fields
		struct Fields<'a, T> {
			$(
				$name:
				
				$( &'a mut [T; $arr_size], )?
				$( &'a mut T, #[cfg(os = "Os that does not exist")] __field: [u8; $val_size], )?
			)*
		}
		
		// Get everything from `array_refs`
		let (
			$(
				$name
			),*
		) = ::arrayref::mut_array_refs!(
			$arr,
			$(
				$( $arr_size )?
				$( $val_size )?
			),*
		);
		
		// And return the fields
		Fields {
			$(
				$name
				$( : &mut ( $name[$val_size - $val_size] ) )?
				,
			)*
		}
	}}
}

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
	
	impl std::error::Error for ReadNullTerminatedStringError {
		fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
			match self {
				Self::NoNull => None,
				Self::Utf8(err) => Some(err),
			}
		}
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
	
	// No source
	impl std::error::Error for WriteNullTerminatedStringError { }
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



/// Error type for [`read_null_ascii_string`]
#[derive(Debug)]
#[derive(derive_more::Display, err_impl::Error)]
pub enum ReadNullAsciiStringError {
	/// No null was found in the string
	#[display(fmt = "No null was found on the buffer")]
	NoNull,
	
	/// The string was not ascii
	#[display(fmt = "The buffer did not contain valid Ascii")]
	NotAscii( #[error(source)] ascii::AsAsciiStrError ),
}

/// Reads a null-terminated ascii string from a buffer.
pub fn read_null_ascii_string(buf: &impl AsRef<[u8]>) -> Result<&ascii::AsciiStr, ReadNullAsciiStringError> {
	// Find the first null and trim the buffer until it
	let buf = buf.as_ref();
	let buf = match buf.iter().position(|&b| b == 0) {
		Some(null_idx) => &buf[0..null_idx],
		None           => return Err( ReadNullAsciiStringError::NoNull ),
	};
	
	// Then convert it from Ascii
	ascii::AsciiStr::from_ascii(buf)
		.map_err(ReadNullAsciiStringError::NotAscii)
}

/// Error type for [`write_null_ascii_string`]
#[derive(Debug)]
#[derive(derive_more::Display, err_impl::Error)]
pub enum WriteNullAsciiStringError {
	/// The input string was too large
	#[display(fmt = "Input string was too large for buffer. ({}+1 / {})", "input_len", "buffer_len")]
	TooLarge {
		input_len : usize,
		buffer_len: usize,
	},
}

/// Writes a null-terminated ascii string to a buffer and returns it
pub fn write_null_ascii_string<'a>(input: &ascii::AsciiStr, buf: &'a mut [u8]) -> Result<&'a mut [u8], WriteNullAsciiStringError> {
	// If the input string doesn't fit into the buffer (excluding the null byte), return Err
	if input.len() >= buf.len() {
		return Err(WriteNullAsciiStringError::TooLarge{ input_len: input.len(), buffer_len: buf.len() });
	}
	
	// Else copy everything over and set the last byte to null
	// Note: We leave all other bytes as they are, no need to set them to 0
	buf[ 0..input.len() ].copy_from_slice( input.as_bytes() );
	buf[ input.len() ] = 0;
	
	// And return Ok with the buffer
	Ok( buf )
}
