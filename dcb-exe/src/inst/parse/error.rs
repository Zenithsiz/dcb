//! Errors

/// Parsing error
#[derive(PartialEq, Clone, Debug, thiserror::Error)]
pub enum ParseError {
	/// Unknown mnemonic
	#[error("Unknown mnemonic")]
	UnknownMnemonic,

	/// Literal was out of range
	#[error("Literal out of range")]
	LiteralOutOfRange,

	/// Invalid arguments
	#[error("Invalid arguments")]
	InvalidArguments,

	/// Relative jump is too far
	#[error("Relative jump is too far")]
	RelativeJumpTooFar,

	/// Unknown label
	#[error("Unknown label")]
	UnknownLabel,

	/// Target is not properly aligned
	#[error("Target is not properly aligned")]
	TargetAlign,
}
