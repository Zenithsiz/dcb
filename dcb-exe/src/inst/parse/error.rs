//! Errors

// Imports
use snailquote::UnescapeError;
use std::{io, num::ParseIntError};

/// Parsing error
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
	/// Unable to read line
	#[error("Unable to read line")]
	ReadLine(#[source] io::Error),

	/// Unterminated string
	#[error("Unterminated string")]
	UnterminatedString,

	/// Expected ',' between arguments
	#[error("Expected ',' between arguments")]
	ExpectedCommaBetweenArgs,

	/// Unable to unescape string
	#[error("Unable to unescape string")]
	StringUnescape(UnescapeError),

	/// Unable to parse literal
	#[error("Unable to parse literal")]
	ParseLiteral(ParseIntError),

	/// Expected register name
	#[error("Expected register name")]
	ExpectedRegister,

	/// Unknown register name
	#[error("Unknown register name")]
	UnknownRegister,

	/// Unterminated register offset
	#[error("Unterminated register offset")]
	UnterminatedRegisterOffset,
}
