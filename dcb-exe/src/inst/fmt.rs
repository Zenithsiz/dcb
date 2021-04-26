//! Instruction formatting
//!
//! See the [`InstFmt`] type for more details.

// Imports
use crate::{inst::Register, Pos};
use dcb_util::SignedHex;
use std::fmt;

/// Instruction display
// TODO: Move `'a` to gat once they are implemented
pub trait InstDisplay<'a> {
	/// Mnemonic type
	type Mnemonic: fmt::Display;

	/// Args type
	type Args: IntoIterator<Item = InstFmtArg<'a>>;

	/// Returns this instruction's mnemonic
	fn mnemonic<Ctx: DisplayCtx>(&'a self, ctx: &Ctx) -> Self::Mnemonic;

	/// Returns all arguments of this instruction
	fn args<Ctx: DisplayCtx>(&'a self, ctx: &Ctx) -> Self::Args;
}

/// Display context
pub trait DisplayCtx {
	/// Label type
	type Label: fmt::Display;

	/// Current position
	fn cur_pos(&self) -> Pos;

	/// Returns any label at `pos`, possibly with an offset
	fn pos_label(&self, pos: Pos) -> Option<(Self::Label, i64)>;
}

/// An formattable argument
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum InstFmtArg<'a> {
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

	/// String
	String(&'a str),
}

impl<'a> InstFmtArg<'a> {
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

	/// Writes this argument
	pub fn write<Ctx: DisplayCtx>(&self, f: &mut fmt::Formatter, ctx: &Ctx) -> Result<(), fmt::Error> {
		match *self {
			// Register offsets with 0 offset are formatted like normal registers
			InstFmtArg::Register(register) | InstFmtArg::RegisterOffset { register, offset: 0 } => {
				write!(f, "{register}")
			},
			InstFmtArg::RegisterOffset { register, offset } => write!(f, "{:#}({register})", SignedHex(offset)),
			// Note: Literals do not go through label lookup
			InstFmtArg::Literal(value) => write!(f, "{:#}", SignedHex(value)),
			InstFmtArg::Target(pos) => match ctx.pos_label(pos) {
				Some((label, 0)) => write!(f, "{label}"),
				Some((label, offset)) => write!(f, "{label}+{:#}", SignedHex(offset)),
				None => write!(f, "{pos}"),
			},
			InstFmtArg::String(s) => write!(f, "\"{}\"", s.escape_debug()),
		}
	}
}
