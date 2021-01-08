//! Jmp register instructions

// Modules
pub mod imm;
pub mod reg;

/// Raw representation
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[derive(derive_more::From)]
pub enum Raw {
	/// Immediate
	Imm(imm::Raw),

	/// Register
	Reg(reg::Raw),
}

/// Jmp register instructions
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[derive(derive_more::Display)]
pub enum Inst {
	/// Immediate
	Imm(imm::Inst),

	/// Register
	Reg(reg::Inst),
}

impl Inst {
	/// Decodes this instruction
	#[must_use]
	pub fn decode(raw: impl Into<Raw>) -> Option<Self> {
		match raw.into() {
			Raw::Imm(raw) => Some(Self::Imm(imm::Inst::decode(raw)?)),
			Raw::Reg(raw) => Some(Self::Reg(reg::Inst::decode(raw)?)),
		}
	}

	/// Encodes this instruction
	#[must_use]
	pub const fn encode(self) -> Raw {
		match self {
			Self::Imm(inst) => Raw::Imm(inst.encode()),
			Self::Reg(inst) => Raw::Reg(inst.encode()),
		}
	}
}
