//! Nop

// Imports
use super::{Decodable, Encodable};
use crate::inst::{basic, DisplayCtx, InstDisplay, InstFmt, InstFmtArg, InstSize, Register};
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

impl<'a> InstDisplay<'a> for Inst {
	type Args = array::IntoIter<InstFmtArg<'a>, 1>;
	type Mnemonic = &'static str;

	fn mnemonic<Ctx: DisplayCtx>(&'a self, _ctx: &Ctx) -> Self::Mnemonic {
		"nop"
	}

	fn args<Ctx: DisplayCtx>(&'a self, _ctx: &Ctx) -> Self::Args {
		let &Self { len } = self;

		let len = i64::try_from(len).expect("Too many nops");
		array::IntoIter::new([InstFmtArg::Literal(len)])
	}
}

impl InstSize for Inst {
	fn size(&self) -> usize {
		4 * self.len
	}
}

impl InstFmt for Inst {
	fn fmt(&self, _pos: crate::Pos, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self.len {
			1 => write!(f, "nop"),
			len => write!(f, "nop {}", len),
		}
	}
}
