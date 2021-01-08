//! Jump register instructions

// Imports
use crate::exe::inst::Register;
use std::fmt;

/// Jmp register instruction kind
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Kind {
	/// Jump
	Jump,

	/// Jump and link
	JumpLink(Register),
}

impl Kind {
	/// Returns this kind's mnemonic
	#[must_use]
	pub const fn mnemonic(self) -> &'static str {
		match self {
			Self::Jump => "jr",
			Self::JumpLink(_) => "jalr",
		}
	}
}

/// Raw representation
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Raw {
	/// Rs
	pub s: u32,

	/// Rd
	pub d: u32,

	/// Func (lower bit)
	pub f: u32,
}

/// Jmp register instructions
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Inst {
	/// Target
	pub target: Register,

	/// Kind
	pub kind: Kind,
}

impl Inst {
	/// Decodes this instruction
	#[must_use]
	pub fn decode(raw: Raw) -> Option<Self> {
		let kind = match raw.f {
			0 => Kind::Jump,
			1 => Kind::JumpLink(Register::new(raw.d)?),
			_ => return None,
		};
		let target = Register::new(raw.s)?;

		Some(Self { target, kind })
	}

	/// Encodes this instruction
	#[must_use]
	pub const fn encode(self) -> Raw {
		let (f, d) = match self.kind {
			Kind::Jump => (0, 0),
			Kind::JumpLink(reg) => (1, reg.idx()),
		};
		let s = self.target.idx();

		Raw { s, d, f }
	}
}

impl fmt::Display for Inst {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let mnemonic = self.kind.mnemonic();
		let target = self.target;

		match self.kind {
			Kind::Jump => write!(f, "{mnemonic} {target}"),
			Kind::JumpLink(reg) => write!(f, "{mnemonic} {target}, {reg}"),
		}
	}
}
