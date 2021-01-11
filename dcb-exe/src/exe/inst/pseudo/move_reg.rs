//! Move register instruction

// Imports
use std::convert::TryInto;

use super::Decodable;
use crate::exe::inst::{basic, InstFmt, InstSize, Register};

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
