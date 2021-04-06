//! Nop

// Imports
use super::Decodable;
use crate::inst::{basic, InstFmt, InstSize, Register};

/// No-op
///
/// Alias for any number of `sll $zr, $zr, 0`.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Inst {
	/// Length of this nop, in instructions
	len: usize,
}

impl Decodable for Inst {
	fn decode(insts: impl Iterator<Item = basic::Inst> + Clone) -> Option<Self> {
		// Get how many nops there are, in a row
		let len = insts
			.take_while(|inst| {
				matches!(
					inst,
					basic::Inst::Shift(basic::shift::Inst::Imm(basic::shift::imm::Inst {
						dst:  Register::Zr,
						lhs:  Register::Zr,
						rhs:  0,
						kind: basic::shift::imm::Kind::LeftLogical,
					}))
				)
			})
			.count();

		match len {
			0 => None,
			_ => Some(Self { len }),
		}
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
