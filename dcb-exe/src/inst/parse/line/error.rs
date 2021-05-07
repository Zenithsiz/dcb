//! Errors

// Imports
use snailquote::UnescapeError;

/// Error for [`Line::parse`](super::Line::parse)
#[derive(PartialEq, Debug, thiserror::Error)]
pub enum ParseLineError {
	/// Unable to parse name
	#[error("Expected name")]
	ParseName(#[from] ParseNameError),

	/// Invalid name suffix
	#[error("Invalid name suffix")]
	InvalidNameSuffix,

	/// Unable to parse argument
	#[error("Expected argument")]
	ParseArg(#[from] ParseArgError),

	/// Invalid argument suffix
	#[error("Invalid argument suffix")]
	InvalidArgSuffix,

	/// Found label after branch delay token
	#[error("Labels must come before the branch delay token")]
	LabelAfterBranchDelay,
}

/// Name parsing error
#[derive(PartialEq, Clone, Debug, thiserror::Error)]
pub enum ParseNameError {
	/// Name was empty
	#[error("Name was empty")]
	Empty,

	/// Invalid starting char
	#[error("Invalid starting character")]
	StartChar,
}

/// Literal parsing error
#[derive(PartialEq, Clone, Debug, thiserror::Error)]
pub enum ParseLiteralError {
	/// Parse
	#[error("Unable to parse literal")]
	Parse(#[from] std::num::ParseIntError),
}

/// Func parsing error
#[derive(PartialEq, Clone, Debug, thiserror::Error)]
pub enum ParseFuncError {
	/// Parse
	#[error("Unknown functions")]
	Unknown,
}

/// Argument parsing error
#[derive(PartialEq, Debug, thiserror::Error)]
pub enum ParseArgError {
	/// Empty
	#[error("Argument was empty")]
	Empty,

	/// Invalid argument
	#[error("Invalid starting char")]
	InvalidStartChar,

	/// Parse Literal
	#[error("Unable to parse literal")]
	Literal(#[source] ParseLiteralError),

	/// Parse mnemonic
	#[error("Unable to parse mnemonic")]
	ParseMnemonic(#[source] ParseNameError),

	/// Parse label
	#[error("Unable to parse label")]
	Label(#[source] ParseNameError),

	/// Parse label offset
	#[error("Unable to parse label offset")]
	LabelOffset(#[source] ParseLiteralError),

	/// Parse label func
	#[error("Unable to parse label func")]
	LabelFunc(#[source] ParseFuncError),

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
