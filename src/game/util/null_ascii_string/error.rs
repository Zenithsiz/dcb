//! Errors

/// Error type for [`read`](super::read)
#[derive(PartialEq, Eq, Clone, Copy, Debug, thiserror::Error)]
pub enum ReadError {
	/// No null was found in the string
	#[error("No null was found on the buffer")]
	NoNull,

	/// The string was not ascii
	#[error("The buffer did not contain valid Ascii")]
	NotAscii(#[source] ascii::AsAsciiStrError),
}

/// Error type for [`write`](super::read)
#[derive(PartialEq, Eq, Clone, Copy, Debug, thiserror::Error)]
#[allow(clippy::missing_docs_in_private_items)]
pub enum WriteError {
	/// The input string was too large
	#[error("Input string was too large for buffer. ({}+1 / {})", input_len, buffer_len)]
	TooLarge { input_len: usize, buffer_len: usize },
}
