//! Alu instructions

// Modules
pub mod imm;
pub mod reg;

// Exports
pub use imm::{AluImmInst, AluImmInstKind, AluImmInstRaw};
pub use reg::{AluRegInst, AluRegInstKind, AluRegInstRaw};

/// Raw representation
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum AluInstRaw {
	/// Immediate
	Imm(AluImmInstRaw),

	/// Register
	Reg(AluRegInstRaw),
}

/// Alu register instructions
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[derive(derive_more::Display)]
pub enum AluInst {
	/// Immediate
	Imm(AluImmInst),

	/// Register
	Reg(AluRegInst),
}

impl AluInst {
	/// Decodes this instruction
	#[must_use]
	pub fn decode(raw: AluInstRaw) -> Option<Self> {
		match raw {
			AluInstRaw::Imm(raw) => Self::Imm(AluImmInst::decode(raw)?),
			AluInstRaw::Reg(raw) => Self::Reg(AluRegInst::decode(raw)?),
		}
	}

	/// Encodes this instruction
	#[must_use]
	pub fn encode(self) -> AluInstRaw {
		match self {
			AluInst::Imm(inst) => AluInstRaw::Imm(inst.encode()),
			AluInst::Reg(inst) => AluInstRaw::Reg(inst.encode()),
		}
	}
}
