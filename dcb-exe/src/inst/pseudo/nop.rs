//! Nop

// Imports
use super::{Decodable, Encodable};
use crate::inst::{
	basic, parse::LineArg, DisplayCtx, InstDisplay, InstFmtArg, InstSize, Parsable, ParseCtx, ParseError, Register,
};
use std::{array, convert::TryFrom};

/// No-op
///
/// Alias for any number of `sll $zr, $zr, 0`.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Inst {
	/// Length of this nop, in instructions
	pub len: usize,
}

impl Inst {
	/// Instruction used by the nop
	pub const INST: basic::Inst = basic::Inst::Shift(basic::shift::Inst::Imm(basic::shift::imm::Inst {
		dst:  Register::Zr,
		lhs:  Register::Zr,
		rhs:  0,
		kind: basic::shift::imm::Kind::LeftLogical,
	}));
}

impl Decodable for Inst {
	fn decode(insts: impl Iterator<Item = basic::Inst> + Clone) -> Option<Self> {
		// Get how many nops there are, in a row
		let len = insts.take_while(|inst| matches!(inst, &Self::INST)).count();

		match len {
			0 => None,
			_ => Some(Self { len }),
		}
	}
}

impl Encodable for Inst {
	type Iterator = impl Iterator<Item = basic::Inst>;

	fn encode(&self) -> Self::Iterator {
		std::iter::repeat(Self::INST).take(self.len)
	}
}

impl<'a> Parsable<'a> for Inst {
	fn parse<Ctx: ?Sized + ParseCtx<'a>>(mnemonic: &'a str, args: &'a [LineArg], ctx: &Ctx) -> Result<Self, ParseError> {
		if mnemonic != "nop" {
			return Err(ParseError::UnknownMnemonic);
		}

		let len = match args {
			[] => 1,
			[LineArg::Expr(len)] => ctx.eval_expr_as(len)?,
			_ => return Err(ParseError::InvalidArguments),
		};

		Ok(Self { len })
	}
}

impl<'a> InstDisplay<'a> for Inst {
	type Mnemonic = &'static str;

	type Args = impl Iterator<Item = InstFmtArg<'a>>;

	fn mnemonic<Ctx: DisplayCtx>(&'a self, _ctx: &Ctx) -> Self::Mnemonic {
		"nop"
	}

	#[auto_enums::auto_enum(Iterator)]
	fn args<Ctx: DisplayCtx>(&'a self, _ctx: &Ctx) -> Self::Args {
		let &Self { len } = self;

		let len = i64::try_from(len).expect("Too many nops");
		match len {
			1 => array::IntoIter::new([]),
			_ => array::IntoIter::new([InstFmtArg::Literal(len)]),
		}
	}
}

impl InstSize for Inst {
	fn size(&self) -> usize {
		4 * self.len
	}
}
