//! Jmp register instructions

// Modules
pub mod imm;
pub mod reg;

// Exports
pub use imm::{JmpImmInst, JmpImmInstRaw};
pub use reg::{JmpRegInst, JmpRegInstRaw};

/// Raw representation
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum JmpInstRaw {
	/// Immediate
	Imm(JmpImmInstRaw),

	/// Register
	Reg(JmpRegInstRaw),
}

/// Jmp register instructions
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[derive(derive_more::Display)]
pub enum JmpInst {
	/// Immediate
	Imm(JmpImmInst),

	/// Register
	Reg(JmpRegInst),
}

impl JmpInst {
	/// Decodes this instruction
	#[must_use]
	pub fn decode(raw: JmpInstRaw) -> Option<Self> {
		match raw {
			JmpInstRaw::Imm(raw) => Self::Imm(JmpImmInst::decode(raw)?),
			JmpInstRaw::Reg(raw) => Self::Reg(JmpRegInst::decode(raw)?),
		}
	}

	/// Encodes this instruction
	#[must_use]
	pub fn encode(self) -> JmpInstRaw {
		match self {
			JmpInst::Imm(inst) => JmpInstRaw::Imm(inst.encode()),
			JmpInst::Reg(inst) => JmpInstRaw::Reg(inst.encode()),
		}
	}
}
