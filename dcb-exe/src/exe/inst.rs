//! Psx cpu instructions
//!
//! This module defines all instructions for the psx cpu, the
//! `MIPS R3051`, following [Nocash's specifications](https://problemkaputt.de/psx-spx.htm).
//!
//! The instructions are split across 3 main types,
//! - `[basic::Inst]`, which defines all 'basic' instructions, i.e. all instructions which are
//!   a single word in size and that carry no simplifications (such as `addi $a0, $a0, 10` == `addi $a0, 10`).
//! - `[pseudo::Inst]`, instructions which are decoded from basic instructions and that represent either
//!   a simplified version of an instruction, or multiple instructions (such as `la $a0, 0x80001000` == `lui $a0, 0x8000 / addiu $ao, 0x1000`).
//! - `[Directive]`, which represent data, rather than instructions, such as `dw` and `.ascii`.
//!
//! See each instruction's module for information on how they are decoded and their variants.
//!
//! Every instruction also uses the [`Register`] enum, which defines all registers in the cpu (except instruction specific
//! registers, such as the `lo` and `hi` registers).
//!
//! This module also contains the [`iter`] module, home to the [`ParseIter`] type, an iterator which parses
//! instructions from raw bytes and their position in memory.

// Modules
pub mod basic;
pub mod directive;
pub mod iter;
pub mod pseudo;
pub mod reg;

// Exports
pub use directive::Directive;
pub use iter::ParseIter;
pub use reg::Register;

// Imports
use crate::Pos;
use std::fmt;

/// An assembler instruction.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Inst {
	/// A basic instruction
	Basic(basic::Inst),

	/// A pseudo instruction
	Pseudo(pseudo::Inst),

	/// A directive
	Directive(Directive),
}

impl Inst {
	/// End of the code itself in the executable.
	pub const CODE_END: Pos = Pos(0x8006dd3c);
	/// Code range
	pub const CODE_RANGE: std::ops::Range<Pos> = Self::CODE_START..Self::CODE_END;
	/// Start of the code itself in the executable.
	pub const CODE_START: Pos = Pos(0x80013e4c);
}

impl InstFmt for Inst {
	fn mnemonic(&self) -> &'static str {
		match self {
			Self::Basic(inst) => inst.mnemonic(),
			Self::Pseudo(inst) => inst.mnemonic(),
			Self::Directive(directive) => directive.mnemonic(),
		}
	}

	fn fmt(&self, pos: Pos, bytes: &[u8], f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Basic(inst) => inst.fmt(pos, bytes, f),
			Self::Pseudo(inst) => inst.fmt(pos, bytes, f),
			Self::Directive(directive) => directive.fmt(pos, bytes, f),
		}
	}
}

/// A formattable basic instruction
///
/// This trait defines formatting for all instruction, which may require the
/// instruction's current position (for relative instructions, such as the
/// branching instructions), as well as the byte array containing the entire
/// executable.
pub trait InstFmt {
	/// Returns this instruction's mnemonic
	fn mnemonic(&self) -> &'static str;

	/// Formats this instruction given it's position and input bytes
	fn fmt(&self, pos: Pos, bytes: &[u8], f: &mut fmt::Formatter) -> fmt::Result;

	/// Returns a wrapped value that may be formatted using [`fmt::Display`]
	fn fmt_value<'a>(&'a self, pos: Pos, bytes: &'a [u8]) -> InstFmtWrapper<Self> {
		InstFmtWrapper { inst: self, pos, bytes }
	}
}

/// Wrapper over [`InstFmt`] values to be displayed using [`fmt::Display`]
#[derive(Clone, Copy, Debug)]
pub struct InstFmtWrapper<'a, T: ?Sized + InstFmt> {
	/// Value
	pub inst: &'a T,

	/// Position
	pub pos: Pos,

	/// Bytes
	pub bytes: &'a [u8],
}

impl<'a, T: ?Sized + InstFmt> fmt::Display for InstFmtWrapper<'a, T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.inst.fmt(self.pos, self.bytes, f)
	}
}
