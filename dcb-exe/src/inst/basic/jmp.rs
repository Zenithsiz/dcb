//! Jmp register instructions

// Modules
pub mod imm;
pub mod reg;

// Imports
use super::ModifiesReg;
use crate::inst::{
	basic::{Decodable, Encodable},
	InstFmt, Register,
};

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
#[derive(derive_more::TryInto)]
pub enum Inst {
	/// Immediate
	Imm(imm::Inst),

	/// Register
	Reg(reg::Inst),
}

impl Decodable for Inst {
	type Raw = Raw;

	fn decode(raw: Self::Raw) -> Option<Self> {
		match raw {
			Raw::Imm(raw) => Some(Self::Imm(imm::Inst::decode(raw)?)),
			Raw::Reg(raw) => Some(Self::Reg(reg::Inst::decode(raw)?)),
		}
	}
}

impl Encodable for Inst {
	fn encode(&self) -> Self::Raw {
		match self {
			Self::Imm(inst) => Raw::Imm(inst.encode()),
			Self::Reg(inst) => Raw::Reg(inst.encode()),
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
