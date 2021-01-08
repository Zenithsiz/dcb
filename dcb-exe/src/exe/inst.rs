//! Psx cpu instructions

// Modules
pub mod basic;
pub mod directive;
pub mod iter;
pub mod pseudo;
pub mod raw;
pub mod reg;

// Exports
pub use directive::Directive;
pub use iter::ParseIter;
pub use raw::Raw;
pub use reg::Register;

// Imports
use crate::Pos;
use std::fmt;

/// An assembler instruction
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
pub trait InstFmt {
	/// Returns this instruction's mnemonic
	fn mnemonic(&self) -> &'static str;

	/// Formats this instruction given it's position and input bytes
	fn fmt(&self, pos: Pos, bytes: &[u8], f: &mut fmt::Formatter) -> fmt::Result;
}

/// Wrapper over [`InstFmt`] values to be displayed using [`fmt::Display`]
#[derive(Clone, Copy, Debug)]
pub struct InstFmtWrapper<'a, T: InstFmt> {
	/// Value
	pub inst: &'a T,

	/// Position
	pub pos: Pos,

	/// Bytes
	pub bytes: &'a [u8],
}

impl<'a, T: InstFmt> fmt::Display for InstFmtWrapper<'a, T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.inst.fmt(self.pos, self.bytes, f)
	}
}
