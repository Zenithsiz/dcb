//! Null-terminated ascii string helpers

// Modules
pub mod error;

// Exports
pub use error::{ReadError, WriteError};

/// Trait for reading null terminated ascii strings from a buffer
pub trait NullAsciiString {
	/// Reads a null terminated ascii string from this buffer and returns it
	fn read_string(&self) -> Result<&ascii::AsciiStr, ReadError>;

	/// Writes a null terminated ascii string to this buffer and returns it
	fn write_string(&mut self, s: &ascii::AsciiStr) -> Result<&Self, WriteError>;
}

impl NullAsciiString for [u8] {
	fn read_string(&self) -> Result<&ascii::AsciiStr, ReadError> {
		// Find the first null and trim the buffer until it
		let buf = match self.iter().position(|&b| b == b'\0') {
			Some(idx) => &self[0..idx],
			None => return Err(ReadError::NoNull),
		};

		// Then convert it from Ascii
		ascii::AsciiStr::from_ascii(buf).map_err(ReadError::NotAscii)
	}

	fn write_string(&mut self, input: &ascii::AsciiStr) -> Result<&Self, WriteError> {
		// If the input string doesn't fit into the buffer (excluding the null byte), return Err
		if input.len() >= self.len() {
			return Err(WriteError::TooLarge {
				input_len:  input.len(),
				buffer_len: self.len(),
			});
		}

		// Else copy everything over and set the last byte to null
		// Note: We leave all other bytes as they are, no need to set them to 0
		self[0..input.len()].copy_from_slice(input.as_bytes());
		self[input.len()] = 0;

		// And return Ok with the buffer
		Ok(self)
	}
}
