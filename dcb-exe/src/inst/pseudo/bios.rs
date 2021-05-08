//! Bios functions


// Imports
use super::{nop, Decodable, Encodable};
use crate::inst::{
	basic, parse::LineArg, DisplayCtx, InstDisplay, InstFmtArg, InstSize, Parsable, ParseCtx, ParseError, Register,
};
use std::{array, convert::TryInto};

/// Bios function kind
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum FuncKind {
	/// A function
	A,

	/// B function
	B,

	/// C function
	C,
}

/// Kind
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Kind {
	/// Jump
	Jump,

	/// Jump and link
	JumpLink,
}

/// Bios call instruction
///
/// Alias for
/// ```mips
/// addiu $t2, $zr, <kind>
/// jr $t2
/// + addiu $t1, $zr, <num>
/// ```
/// or
/// ```mips
/// addiu $t1, $zr, <num>
/// addiu $t2, $zr, <kind>
/// jalr $t2
/// + nop
/// ```
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Inst {
	/// Kind
	kind: Kind,

	/// Function
	func: FuncKind,

	/// Function number
	num: u8,
}


impl Decodable for Inst {
	fn decode(mut insts: impl Iterator<Item = basic::Inst> + Clone) -> Option<Self> {
		let (kind, func, num) = match (insts.next()?, insts.next()?, insts.next()?, insts.next()) {
			(
				basic::Inst::Alu(basic::alu::Inst::Imm(basic::alu::imm::Inst {
					dst: Register::T2,
					lhs: Register::Zr,
					kind: basic::alu::imm::Kind::AddUnsigned(func),
				})),
				basic::Inst::Jmp(basic::jmp::Inst::Reg(basic::jmp::reg::Inst {
					target: Register::T2,
					kind: basic::jmp::reg::Kind::Jump,
				})),
				basic::Inst::Alu(basic::alu::Inst::Imm(basic::alu::imm::Inst {
					dst: Register::T1,
					lhs: Register::Zr,
					kind: basic::alu::imm::Kind::AddUnsigned(num),
				})),
				_,
			) => (Kind::Jump, func, num),

			(
				basic::Inst::Alu(basic::alu::Inst::Imm(basic::alu::imm::Inst {
					dst: Register::T1,
					lhs: Register::Zr,
					kind: basic::alu::imm::Kind::AddUnsigned(num),
				})),
				basic::Inst::Alu(basic::alu::Inst::Imm(basic::alu::imm::Inst {
					dst: Register::T2,
					lhs: Register::Zr,
					kind: basic::alu::imm::Kind::AddUnsigned(func),
				})),
				basic::Inst::Jmp(basic::jmp::Inst::Reg(basic::jmp::reg::Inst {
					target: Register::T2,
					kind: basic::jmp::reg::Kind::JumpLink(Register::Ra),
				})),
				Some(nop::Inst::INST),
			) => (Kind::JumpLink, func, num),

			_ => return None,
		};

		Some(Self {
			kind,
			func: match func {
				0xa0 => FuncKind::A,
				0xb0 => FuncKind::B,
				0xc0 => FuncKind::C,
				_ => return None,
			},
			num: num.try_into().ok()?,
		})
	}
}

impl Encodable for Inst {
	type Iterator = impl Iterator<Item = basic::Inst>;

	#[auto_enums::auto_enum(Iterator)]
	fn encode(&self) -> Self::Iterator {
		let func = match self.func {
			FuncKind::A => 0xa0,
			FuncKind::B => 0xb0,
			FuncKind::C => 0xc0,
		};

		match self.kind {
			Kind::Jump => array::IntoIter::new([
				basic::Inst::Alu(basic::alu::Inst::Imm(basic::alu::imm::Inst {
					dst:  Register::T2,
					lhs:  Register::Zr,
					kind: basic::alu::imm::Kind::AddUnsigned(func),
				})),
				basic::Inst::Jmp(basic::jmp::Inst::Reg(basic::jmp::reg::Inst {
					target: Register::T2,
					kind:   basic::jmp::reg::Kind::Jump,
				})),
				basic::Inst::Alu(basic::alu::Inst::Imm(basic::alu::imm::Inst {
					dst:  Register::T1,
					lhs:  Register::Zr,
					kind: basic::alu::imm::Kind::AddUnsigned(self.num.into()),
				})),
			]),
			Kind::JumpLink => array::IntoIter::new([
				basic::Inst::Alu(basic::alu::Inst::Imm(basic::alu::imm::Inst {
					dst:  Register::T1,
					lhs:  Register::Zr,
					kind: basic::alu::imm::Kind::AddUnsigned(self.num.into()),
				})),
				basic::Inst::Alu(basic::alu::Inst::Imm(basic::alu::imm::Inst {
					dst:  Register::T2,
					lhs:  Register::Zr,
					kind: basic::alu::imm::Kind::AddUnsigned(func),
				})),
				basic::Inst::Jmp(basic::jmp::Inst::Reg(basic::jmp::reg::Inst {
					target: Register::T2,
					kind:   basic::jmp::reg::Kind::JumpLink(Register::Ra),
				})),
				nop::Inst::INST,
			]),
		}
	}
}

impl<'a> Parsable<'a> for Inst {
	fn parse<Ctx: ?Sized + ParseCtx<'a>>(
		mnemonic: &'a str, args: &'a [LineArg], ctx: &Ctx,
	) -> Result<Self, ParseError> {
		let (kind, func) = match mnemonic {
			"jba" => (Kind::Jump, FuncKind::A),
			"jbb" => (Kind::Jump, FuncKind::B),
			"jbc" => (Kind::Jump, FuncKind::C),
			"jalba" => (Kind::JumpLink, FuncKind::A),
			"jalbb" => (Kind::JumpLink, FuncKind::B),
			"jalbc" => (Kind::JumpLink, FuncKind::C),
			_ => return Err(ParseError::UnknownMnemonic),
		};

		let num = match args {
			[LineArg::Expr(num)] => ctx.eval_expr_as(num)?,
			_ => return Err(ParseError::InvalidArguments),
		};

		Ok(Self { kind, func, num })
	}
}

impl<'a> InstDisplay<'a> for Inst {
	type Args = array::IntoIter<InstFmtArg<'a>, 1>;
	type Mnemonic = &'static str;

	#[rustfmt::skip]
	fn mnemonic<Ctx: DisplayCtx>(&'a self, _ctx: &Ctx) -> Self::Mnemonic {
		match (self.kind, self.func) {
			(Kind::Jump    , FuncKind::A) => "jba",
			(Kind::Jump    , FuncKind::B) => "jbb",
			(Kind::Jump    , FuncKind::C) => "jbc",
			(Kind::JumpLink, FuncKind::A) => "jalba",
			(Kind::JumpLink, FuncKind::B) => "jalbb",
			(Kind::JumpLink, FuncKind::C) => "jalbc",
		}
	}

	fn args<Ctx: DisplayCtx>(&'a self, _ctx: &Ctx) -> Self::Args {
		array::IntoIter::new([InstFmtArg::literal(self.num)])
	}
}

impl InstSize for Inst {
	fn size(&self) -> usize {
		match self.kind {
			Kind::Jump => 12,
			Kind::JumpLink => 16,
		}
	}
}
