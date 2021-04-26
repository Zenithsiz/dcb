//! Instruction formatting
//!
//! See the [`InstFmt`] type for more details.

// Imports
use super::InstTarget;
use crate::{inst::Register, Pos};
use std::fmt;

/// Instruction display
pub trait InstDisplay {
	/// Mnemonic type
	type Mnemonic: fmt::Display;

	/// Args type
	type Args: IntoIterator<Item = InstFmtArg>;

	/// Returns this instruction's mnemonic
	fn mnemonic<Ctx: DisplayCtx>(&self, ctx: &Ctx) -> Self::Mnemonic;

	/// Returns all arguments of this instruction
	fn args<Ctx: DisplayCtx>(&self, ctx: &Ctx) -> Self::Args;
}

/// Display context
pub trait DisplayCtx {
	/// Current position
	fn cur_pos(&self) -> Pos;
}

/// An formattable argument
#[derive(PartialEq, Clone, Debug)]
pub enum InstFmtArg {
	/// Register
	Register(Register),

	/// Register offset
	RegisterOffset {
		/// The register
		register: Register,

		/// The offset
		offset: i64,
	},

	/// Literal
	Literal(i64),

	/// Target
	Target(Pos),
}

impl InstFmtArg {
	/// Creates a `Literal` variant
	#[must_use]
	pub fn literal(value: impl Into<i64>) -> Self {
		Self::Literal(value.into())
	}

	/// Creates a `RegisterOffset` variant
	#[must_use]
	pub fn register_offset(register: Register, offset: impl Into<i64>) -> Self {
		Self::RegisterOffset {
			register,
			offset: offset.into(),
		}
	}
}


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
