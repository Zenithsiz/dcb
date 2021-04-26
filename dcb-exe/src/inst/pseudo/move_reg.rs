//! Move register instruction

// Imports
use super::{Decodable, Encodable};
use crate::inst::{basic, DisplayCtx, InstDisplay, InstFmt, InstFmtArg, InstSize, Register};
use std::{array, convert::TryInto};

/// Move register instruction
///
/// Alias for
/// ```mips
/// addu $dst, $src, $zr
/// ```
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Inst {
	/// Destination register
	pub dst: Register,

	/// Source register
	pub src: Register,
}


impl Decodable for Inst {
	fn decode(mut insts: impl Iterator<Item = basic::Inst> + Clone) -> Option<Self> {
		match insts.next()?.try_into().ok()? {
			basic::alu::Inst::Reg(basic::alu::reg::Inst {
				dst,
				lhs,
				rhs: Register::Zr,
				kind: basic::alu::reg::Kind::AddUnsigned,
			}) => Some(Self { dst, src: lhs }),
			_ => None,
		}
	}
}

impl Encodable for Inst {
	type Iterator = impl Iterator<Item = basic::Inst>;

	fn encode(&self) -> Self::Iterator {
		std::iter::once(basic::Inst::Alu(basic::alu::Inst::Reg(basic::alu::reg::Inst {
			dst:  self.dst,
			lhs:  self.src,
			rhs:  Register::Zr,
			kind: basic::alu::reg::Kind::AddUnsigned,
		})))
	}
}

impl<'a> InstDisplay<'a> for Inst {
	type Args = array::IntoIter<InstFmtArg<'a>, 2>;
	type Mnemonic = &'static str;

	fn mnemonic<Ctx: DisplayCtx>(&'a self, _ctx: &Ctx) -> Self::Mnemonic {
		"move"
	}

	fn args<Ctx: DisplayCtx>(&'a self, _ctx: &Ctx) -> Self::Args {
		let &Self { dst, src } = self;

		array::IntoIter::new([InstFmtArg::Register(dst), InstFmtArg::Register(src)])
	}
}

impl InstSize for Inst {
	fn size(&self) -> usize {
		4
	}
}

impl InstFmt for Inst {
	fn fmt(&self, _pos: crate::Pos, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let Self { dst, src } = self;

		write!(f, "move {dst}, {src}")
	}
}
