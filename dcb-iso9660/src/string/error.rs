//! Errors

/// Error for [`Alphabet::validate`](super::Alphabet::validate)'s impl of [`AlphabetA`](super::AlphabetA) and
/// [`AlphabetD`](super::AlphabetD)
#[derive(Debug, thiserror::Error)]
#[error("Invalid character '{byte:#x}' at index {pos}")]
pub struct InvalidCharError {
	/// Invalid character
	pub byte: u8,

	/// Position
	pub pos: usize,
}

/// Error for [`Alphabet::validate`](super::Alphabet::validate)'s impl of [`AlphabetA`](super::AlphabetA) and
/// [`AlphabetD`](super::AlphabetD)
#[derive(Debug, thiserror::Error)]
pub enum ValidateFileAlphabetError {
	/// Invalid name character
	#[error("Invalid name character")]
	InvalidNameChar(#[source] InvalidCharError),

	/// Invalid extension character
	#[error("Invalid extension character")]
	InvalidExtensionChar(#[source] InvalidCharError),

	/// Missing file name extension
	#[error("Missing file name extension")]
	MissingExtension,

	/// Missing file name version
	#[error("Missing file name version")]
	MissingVersion,

	/// Invalid version
	#[error("Invalid version")]
	InvalidVersion,
}
