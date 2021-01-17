//! Errors

/// Error for [`StringArrA::from_bytes`] and [`StringArrD::from_bytes`]
#[derive(Debug, thiserror::Error)]
#[error("Invalid character '{byte:#x}' at index {pos}")]
pub struct InvalidCharError {
	/// Invalid character
	pub byte: u8,

	/// Position
	pub pos: usize,
}
