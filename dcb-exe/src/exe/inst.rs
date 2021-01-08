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

/// An assembler instruction
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Inst {
	/// A basic instruction
	Basic(basic::Inst),

	/// A pseudo instruction
	Pseudo(pseudo::PseudoInst),

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
