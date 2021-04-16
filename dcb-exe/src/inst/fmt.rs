//! Instruction formatting
//!
//! See the [`InstFmt`] type for more details.

// Imports
use super::InstTarget;
use crate::Pos;
use std::fmt;

/// A formattable instruction
///
/// This trait defines formatting for all instruction, which may require the
/// instruction's current position (for relative instructions, such as the
/// branching instructions).
pub trait InstFmt {
	/// Formats this instruction
	fn fmt(&self, pos: Pos, f: &mut fmt::Formatter) -> fmt::Result;
}

/// A formattable instruction that supports overloading it's target.
pub trait InstTargetFmt {
	/// Formats this instruction
	fn fmt(&self, pos: Pos, target: impl fmt::Display, f: &mut fmt::Formatter) -> fmt::Result;
}

impl<T: InstTarget + InstTargetFmt> InstFmt for T {
	fn fmt(&self, pos: Pos, f: &mut fmt::Formatter) -> fmt::Result {
		let target = self.target(pos);
		self.fmt(pos, target, f)
	}
}
