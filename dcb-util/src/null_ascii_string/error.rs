//! Errors

/// Error type for [`NullAsciiString::read_string`](super::NullAsciiString::read_string)
#[derive(PartialEq, Eq, Clone, Copy, Debug, thiserror::Error)]
pub enum ReadError {
	/// No null was found in the string
	#[error("No null was found on the buffer")]
	NoNull,

	/// The string was not ascii
	#[error("The buffer did not contain valid Ascii")]
	NotAscii(#[source] ascii::AsAsciiStrError),
}
