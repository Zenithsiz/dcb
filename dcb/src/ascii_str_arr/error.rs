//! Errors

/// Error returned when a string was too long to be converted
#[derive(Debug, thiserror::Error)]
#[error("String was too long (max is {} characters)", LEN)]
pub struct TooLongError<const LEN: usize>;

/// Error returned when an input string contained non-ascii characters
#[derive(Debug, thiserror::Error)]
#[error("String contained non-ascii characters (first found at {pos})")]
pub struct NotAsciiError {
	/// Index that contained the first non-ascii character
	pub pos: usize,
}

/// Error returned when converting a `&[u8]` to a `AsciiStrArr`
#[derive(Debug, thiserror::Error)]
pub enum FromBytesError<const LEN: usize> {
	/// Too long
	#[error("String was too long")]
	TooLong(TooLongError<LEN>),

	/// Not ascii
	#[error("String contained non-ascii characters")]
	NotAscii(ascii::AsAsciiStrError),
}
