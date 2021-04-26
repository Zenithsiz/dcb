//! Shift instruction

// Modules
pub mod imm;
pub mod reg;

// Imports
use super::ModifiesReg;
use crate::inst::{
	basic::{Decodable, Encodable},
	parse::LineArg,
	InstFmt, Parsable, ParseCtx, ParseError, Register,
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
	fn decode(raw: u32) -> Option<Self> {
		None.or_else(|| imm::Inst::decode(raw).map(Self::Imm))
			.or_else(|| reg::Inst::decode(raw).map(Self::Reg))
	}
}

impl Encodable for Inst {
	fn encode(&self) -> u32 {
		match self {
			Self::Imm(inst) => inst.encode(),
			Self::Reg(inst) => inst.encode(),
		}
	}
}

impl Parsable for Inst {
	fn parse<Ctx: ?Sized + ParseCtx>(mnemonic: &str, args: &[LineArg], ctx: &Ctx) -> Result<Self, ParseError> {
		match imm::Inst::parse(mnemonic, args, ctx) {
			Ok(inst) => Ok(Self::Imm(inst)),
			Err(ParseError::UnknownMnemonic) => reg::Inst::parse(mnemonic, args, ctx).map(Self::Reg),
			Err(err) => Err(err),
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
