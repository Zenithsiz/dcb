//! Instruction parsing

// Modules
pub mod error;
pub mod line;

// Exports
pub use error::ParseError;
pub use line::{Line, LineArg, LineInst, LineLabel};

// Imports
use crate::Pos;
use std::convert::TryInto;

/// Instruction parsing
pub trait Parsable: Sized {
	/// Parses this instruction
	fn parse<Ctx: ?Sized + ParseCtx>(mnemonic: &str, args: &[LineArg], ctx: &Ctx) -> Result<Self, ParseError>;
}

/// Parsing context
pub trait ParseCtx {
	/// Returns the current position
	fn cur_pos(&self) -> Pos;

	/// Returns the position of a label
	fn label_pos(&self, label: &str) -> Option<Pos>;

	/// Retrieves a position from an argument
	fn arg_pos(&self, arg: &LineArg) -> Result<Pos, ParseError> {
		match *arg {
			LineArg::Literal(pos) => pos.try_into().map(Pos).map_err(|_| ParseError::LiteralOutOfRange),
			LineArg::Label(ref label) => self.label_pos(label).ok_or(ParseError::UnknownLabel),
			_ => Err(ParseError::InvalidArguments),
		}
	}

	/// Retrieves a position and offset from an argument
	fn arg_pos_offset(&self, arg: &LineArg) -> Result<(Pos, i64), ParseError> {
		match *arg {
			LineArg::Literal(pos) => pos.try_into().map(|pos| (Pos(pos), 0)).map_err(|_| ParseError::LiteralOutOfRange),
			LineArg::Label(ref label) => self.label_pos(label).map(|pos| (pos, 0)).ok_or(ParseError::UnknownLabel),
			LineArg::LabelOffset { ref label, offset } => self.label_pos(label).map(|pos| (pos, offset)).ok_or(ParseError::UnknownLabel),
			_ => Err(ParseError::InvalidArguments),
		}
	}
}
