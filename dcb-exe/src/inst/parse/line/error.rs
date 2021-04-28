//! Errors

use snailquote::UnescapeError;


/// Error for [`Line::parse`](super::Line::parse)
#[derive(Debug, thiserror::Error)]
pub enum ParseLineError {
	/// Unable to read name
	#[error("Expected name")]
	ReadName(#[from] ReadNameError),

	/// Invalid name suffix
	#[error("Invalid name suffix")]
	InvalidNameSuffix,

	/// Unable to read argument
	#[error("Expected argument")]
	ReadArg(#[from] ReadArgError),

	/// Invalid argument suffix
	#[error("Invalid argument suffix")]
	InvalidArgSuffix,
}

/// Name reading error
#[derive(Debug, thiserror::Error)]
pub enum ReadNameError {
	/// Name was empty
	#[error("Name was empty")]
	Empty,

	/// Invalid starting char
	#[error("Invalid starting character")]
	StartChar,
}

/// Literal reading error
#[derive(Debug, thiserror::Error)]
pub enum ReadLiteralError {
	/// Parse
	#[error("Unable to parse literal")]
	Parse(#[from] std::num::ParseIntError),
}

/// Func reading error
#[derive(Debug, thiserror::Error)]
pub enum ReadFuncError {
	/// Parse
	#[error("Unknown functions")]
	Unknown,
}

/// Argument reading error
#[derive(Debug, thiserror::Error)]
pub enum ReadArgError {
	/// Empty
	#[error("Argument was empty")]
	Empty,

	/// Invalid argument
	#[error("Invalid starting char")]
	InvalidStartChar,

	/// Read Literal
	#[error("Unable to read literal")]
	ReadLiteral(#[source] ReadLiteralError),

	/// Read mnemonic
	#[error("Unable to read mnemonic")]
	ReadMnemonic(#[source] ReadNameError),

	/// Read label
	#[error("Unable to read label")]
	ReadLabel(#[source] ReadNameError),

	/// Read label offset
	#[error("Unable to read label offset")]
	ReadLabelOffset(#[source] ReadLiteralError),

	/// Read label func
	#[error("Unable to read label func")]
	ReadLabelFunc(#[source] ReadFuncError),

	/// Expected register
	#[error("Expected register")]
	ExpectedRegister,

	/// Unknown register
	#[error("Unknown register")]
	UnknownRegister,

	/// Missing ')' for register offset
	#[error("Missing ')' for register offset")]
	MissingRegisterOffsetDelimiter,

	/// Leftover tokens in register offset
	#[error("Leftover tokens in register offset")]
	RegisterOffsetLeftoverTokens,

	/// Missing closing '"' for string
	#[error("Missing closing delimiter for string")]
	MissingClosingDelimiterString,

	/// Unable to unescape string
	#[error("Unable to unescape string")]
	UnescapeString(UnescapeError),
}
