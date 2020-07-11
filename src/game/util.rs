//! Utility macros and functions
//!
//! This modules is used for miscellaneous macros, functions that have
//! not been moved to a more permanent location.
//!
//! All items in this module will eventually be deprecated and moved
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
		#![allow(
			clippy::used_underscore_binding,
			clippy::ptr_offset_with_cast,
			clippy::indexing_slicing,
		)]

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
		#![allow(
			clippy::used_underscore_binding,
			clippy::ptr_offset_with_cast,
			clippy::indexing_slicing,
		)]

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

/// Error type for [`read_null_ascii_string`]
#[derive(PartialEq, Eq, Clone, Copy, Debug, thiserror::Error)]
pub enum ReadNullAsciiStringError {
	/// No null was found in the string
	#[error("No null was found on the buffer")]
	NoNull,

	/// The string was not ascii
	#[error("The buffer did not contain valid Ascii")]
	NotAscii(#[source] ascii::AsAsciiStrError),
}

/// Reads a null-terminated ascii string from a buffer.
pub fn read_null_ascii_string(buf: &impl AsRef<[u8]>) -> Result<&ascii::AsciiStr, ReadNullAsciiStringError> {
	// Find the first null and trim the buffer until it
	let buf = buf.as_ref();
	let buf = match buf.iter().position(|&b| b == 0) {
		Some(null_idx) => &buf[0..null_idx],
		None => return Err(ReadNullAsciiStringError::NoNull),
	};

	// Then convert it from Ascii
	ascii::AsciiStr::from_ascii(buf).map_err(ReadNullAsciiStringError::NotAscii)
}

/// Error type for [`read_maybe_null_ascii_string`]
#[derive(PartialEq, Eq, Clone, Copy, Debug, thiserror::Error)]
pub enum ReadMaybeNullAsciiStringError {
	/// The string was not ascii
	#[error("The buffer did not contain valid Ascii")]
	NotAscii(#[source] ascii::AsAsciiStrError),
}

/// Reads a possibly null-terminated ascii string from a buffer.
pub fn read_maybe_null_ascii_string(buf: &impl AsRef<[u8]>) -> Result<&ascii::AsciiStr, ReadMaybeNullAsciiStringError> {
	// Find the first null and trim the buffer until it
	let buf = buf.as_ref();
	let buf = match buf.iter().position(|&b| b == 0) {
		Some(null_idx) => &buf[0..null_idx],
		None => buf,
	};

	// Then convert it from Ascii
	ascii::AsciiStr::from_ascii(buf).map_err(ReadMaybeNullAsciiStringError::NotAscii)
}

/// Error type for [`write_null_ascii_string`]
#[derive(PartialEq, Eq, Clone, Copy, Debug, thiserror::Error)]
pub enum WriteNullAsciiStringError {
	/// The input string was too large
	#[error("Input string was too large for buffer. ({}+1 / {})", "input_len", "buffer_len")]
	TooLarge { input_len: usize, buffer_len: usize },
}

/// Writes a null-terminated ascii string to a buffer and returns it
pub fn write_null_ascii_string<'a>(input: &ascii::AsciiStr, buf: &'a mut [u8]) -> Result<&'a mut [u8], WriteNullAsciiStringError> {
	// If the input string doesn't fit into the buffer (excluding the null byte), return Err
	if input.len() >= buf.len() {
		return Err(WriteNullAsciiStringError::TooLarge {
			input_len:  input.len(),
			buffer_len: buf.len(),
		});
	}

	// Else copy everything over and set the last byte to null
	// Note: We leave all other bytes as they are, no need to set them to 0
	buf[0..input.len()].copy_from_slice(input.as_bytes());
	buf[input.len()] = 0;

	// And return Ok with the buffer
	Ok(buf)
}

/// Error type for [`write_maybe_null_ascii_string`]
#[derive(PartialEq, Eq, Clone, Copy, Debug, thiserror::Error)]
pub enum WriteMaybeNullAsciiStringError {
	/// The input string was too large
	#[error("Input string was too large for buffer. ({} / {})", "input_len", "buffer_len")]
	TooLarge { input_len: usize, buffer_len: usize },
}

/// Writes a possibly null-terminated ascii string to a buffer and returns it
pub fn write_maybe_null_ascii_string<'a>(input: &ascii::AsciiStr, buf: &'a mut [u8]) -> Result<&'a mut [u8], WriteMaybeNullAsciiStringError> {
	// If the input string doesn't fit into the buffer, return Err
	if input.len() > buf.len() {
		return Err(WriteMaybeNullAsciiStringError::TooLarge {
			input_len:  input.len(),
			buffer_len: buf.len(),
		});
	}

	// Copy everything over to the slice
	// Note: We leave all other bytes as they are, no need to set them to 0
	buf[0..input.len()].copy_from_slice(input.as_bytes());

	// If there's a character left, write it to null
	if let Some(null_byte) = buf.get_mut(input.len()) {
		*null_byte = 0;
	}

	// And return Ok with the buffer
	Ok(buf)
}
