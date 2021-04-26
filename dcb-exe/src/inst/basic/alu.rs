//! Alu instructions

// Modules
pub mod imm;
pub mod reg;

// Imports
use super::ModifiesReg;
use crate::inst::{
	basic::{Decode, Encode},
	parse::LineArg,
	DisplayCtx, InstDisplay, InstFmt, InstFmtArg, Parsable, ParseCtx, ParseError,
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

impl Decode for Inst {
	fn decode(raw: u32) -> Option<Self> {
		None.or_else(|| imm::Inst::decode(raw).map(Self::Imm))
			.or_else(|| reg::Inst::decode(raw).map(Self::Reg))
	}
}

impl Encode for Inst {
	fn encode(&self) -> u32 {
		match self {
			Inst::Imm(inst) => inst.encode(),
			Inst::Reg(inst) => inst.encode(),
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

impl<'a> InstDisplay<'a> for Inst {
	type Mnemonic = &'static str;

	type Args = impl Iterator<Item = InstFmtArg<'a>>;

	fn mnemonic<Ctx: DisplayCtx>(&'a self, ctx: &Ctx) -> Self::Mnemonic {
		match self {
			Inst::Imm(inst) => inst.mnemonic(ctx),
			Inst::Reg(inst) => inst.mnemonic(ctx),
		}
	}

	#[auto_enums::auto_enum(Iterator)]
	fn args<Ctx: DisplayCtx>(&'a self, ctx: &Ctx) -> Self::Args {
		match self {
			Inst::Imm(inst) => inst.args(ctx).into_iter(),
			Inst::Reg(inst) => inst.args(ctx).into_iter(),
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
	fn modifies_reg(&self, reg: crate::inst::Register) -> bool {
		match self {
			Inst::Imm(inst) => inst.modifies_reg(reg),
			Inst::Reg(inst) => inst.modifies_reg(reg),
		}
	}
}
