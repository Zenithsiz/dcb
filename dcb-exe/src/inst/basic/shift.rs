//! Shift instruction

// Modules
pub mod imm;
pub mod reg;

// Imports
use super::ModifiesReg;
use crate::inst::{
	basic::{Decodable, Encodable},
	InstFmt, Register,
};

/// Alu register instructions
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[derive(derive_more::TryInto)]
pub enum Inst {
	/// Immediate
	Imm(imm::Inst),

	/// Register
	Reg(reg::Inst),
}

impl Decodable for Inst {
	type Raw = u32;

	fn decode(raw: Self::Raw) -> Option<Self> {
		None.or_else(|| imm::Inst::decode(raw).map(Self::Imm))
			.or_else(|| reg::Inst::decode(raw).map(Self::Reg))
	}
}

impl Encodable for Inst {
	fn encode(&self) -> Self::Raw {
		match self {
			Self::Imm(inst) => inst.encode(),
			Self::Reg(inst) => inst.encode(),
		}
	}
}

impl InstFmt for Inst {
	fn fmt(&self, pos: crate::Pos, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Self::Imm(inst) => inst.fmt(pos, f),
			Self::Reg(inst) => inst.fmt(pos, f),
		}
	}
}

impl ModifiesReg for Inst {
	fn modifies_reg(&self, reg: Register) -> bool {
		match self {
			Inst::Imm(inst) => inst.modifies_reg(reg),
			Inst::Reg(inst) => inst.modifies_reg(reg),
		}
	}
}
