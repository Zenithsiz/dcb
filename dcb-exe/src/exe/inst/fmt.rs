//! Instruction formatting
//!
//! See the [`InstFmt`] type for more details.

// Imports
use crate::Pos;
use std::fmt;

/// A formattable basic instruction
///
/// This trait defines formatting for all instruction, which may require the
/// instruction's current position (for relative instructions, such as the
/// branching instructions), as well as the byte array containing the entire
/// executable.
pub trait InstFmt {
	/// Formats this instruction
	fn fmt(&self, pos: Pos, f: &mut fmt::Formatter) -> fmt::Result;

	/// Returns a wrapped value that may be formatted using [`fmt::Display`]
	fn fmt_value(&self, pos: Pos) -> InstFmtWrapper<Self> {
		InstFmtWrapper { inst: self, pos }
	}
}

/// Wrapper over [`InstFmt`] values to be displayed using [`fmt::Display`]
#[derive(Clone, Copy, Debug)]
pub struct InstFmtWrapper<'a, T: ?Sized + InstFmt> {
	/// Value
	pub inst: &'a T,

	/// Position
	pub pos: Pos,
}

impl<'a, T: ?Sized + InstFmt> fmt::Display for InstFmtWrapper<'a, T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.inst.fmt(self.pos, f)
	}
}
