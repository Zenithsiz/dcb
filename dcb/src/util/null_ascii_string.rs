//! Null-terminated ascii string helpers

// Modules
pub mod error;

// Exports
pub use error::ReadError;

// Imports
use crate::AsciiStrArr;
use std::convert::TryInto;

/// Trait for reading null terminated ascii strings from a buffer
pub trait NullAsciiString<const N: usize> {
	/// Reads a null terminated ascii string from this buffer and returns it
	fn read_string(&self) -> Result<AsciiStrArr<N>, ReadError>;

	/// Writes a null terminated ascii string to this buffer and returns it
	fn write_string(&mut self, s: &AsciiStrArr<N>);
}

// TODO: Get rid of this once we're able to use `{N + 1}`
macro impl_null_ascii_string($($N:expr),* $(,)?) {
	$(
		impl NullAsciiString<$N> for [u8; $N + 1] {
			fn read_string(&self) -> Result<AsciiStrArr<$N>, ReadError> {
				// Find the first null and trim the buffer until it
				let buf = match self.iter().position(|&b| b == b'\0') {
					// SAFETY: `idx` is guaranteed to be less than our length
					Some(idx) => unsafe { self.get_unchecked(..idx) },
					None => return Err(ReadError::NoNull),
				};

				// Then convert it to the ascii string array
				Ok(ascii::AsciiStr::from_ascii(buf)
					.map_err(ReadError::NotAscii)?
					.try_into()
					.expect("Null terminated `[u8; N+1]` didn't fit into `AsciiStringArr<N>`")
				)
			}

			#[allow(unused_comparisons)] // With N = 0 this function does nothing
			fn write_string(&mut self, input: &AsciiStrArr<$N>) {
				// Copy everything over and set the last byte to 0
				// SAFETY: We guarantee `len < N`.
				// Note: No need to override the remaining bytes
				let len = input.len();
				debug_assert!(len < $N);
				unsafe { std::intrinsics::copy_nonoverlapping::<u8>(input.as_bytes().as_ptr(), self.as_mut_ptr(), len) };
				*unsafe { self.get_unchecked_mut(len) } = 0;
			}
		}
	)*
}

#[rustfmt::skip]
impl_null_ascii_string!(
	 0,  1,  2,  3,  4,  5,  6,  7,  8,  9,
	10, 11, 12, 13, 14, 15, 16, 17, 18, 19,
	20, 21, 22, 23, 24, 25, 26, 27, 28, 29,
	30, 31, 32,
);
