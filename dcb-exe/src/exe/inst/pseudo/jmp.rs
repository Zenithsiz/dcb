//! Jump pseudo instructions

// Imports
use crate::exe::inst::{basic, InstFmt, InstSize, Register};

use super::Decodable;

/// Jump / Branch instructions
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Inst {
	/// Jump and link with return address
	/// Alias for `jalr $target, $ra`
	JalrRa {
		/// Target for the jump
		target: Register,
	},

	/// Branch if equal to zero
	/// Alias for `beq $arg, $zr, offset`
	Beqz {
		/// Argument to compare
		arg: Register,

		/// Jump offset
		offset: i16,
	},

	/// Branch if different from zero
	/// Alias for `bne $arg, $zr, offset`
	Bnez {
		/// Argument to compare
		arg: Register,

		/// Jump offset
		offset: i16,
	},

	/// Jump relative
	/// Alias for `beq $zr, $zr, offset`
	B {
		/// Jump offset
		offset: i16,
	},
}

impl Decodable for Inst {
	fn decode(mut insts: impl Iterator<Item = basic::Inst> + Clone) -> Option<Self> {
		let inst = match insts.next()? {
			basic::Inst::Cond(basic::cond::Inst { arg, offset, kind }) => match kind {
				basic::cond::Kind::Equal(Register::Zr) => match arg {
					// `beq $zr, $zr, offset`
					Register::Zr => Self::B { offset },
					// `beq $zr, $arg, offset`
					_ => Self::Beqz { arg, offset },
				},
				// `bnq $zr, $arg, offset`
				basic::cond::Kind::NotEqual(Register::Zr) => Self::Bnez { arg, offset },
				_ => return None,
			},
			// `jalr $ra, $target`
			basic::Inst::Jmp(basic::jmp::Inst::Reg(basic::jmp::reg::Inst {
				target,
				kind: basic::jmp::reg::Kind::JumpLink(Register::Ra),
			})) => Self::JalrRa { target },

			_ => return None,
		};

		Some(inst)
	}
}

impl InstSize for Inst {
	fn size(&self) -> usize {
		4
	}
}

impl InstFmt for Inst {
	fn mnemonic(&self) -> &'static str {
		match self {
			Self::JalrRa { .. } => "jalr",
			Self::Beqz { .. } => "beqz",
			Self::Bnez { .. } => "bnez",
			Self::B { .. } => "b",
		}
	}

	fn fmt(&self, pos: crate::Pos, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let mnemonic = self.mnemonic();

		match *self {
			Self::JalrRa { target } => write!(f, "{mnemonic} {target}"),
			Self::Beqz { arg, offset } | Self::Bnez { arg, offset } => write!(f, "{mnemonic} {arg}, {}", basic::cond::Inst::target_of(offset, pos)),
			Self::B { offset } => write!(f, "{mnemonic} {}", basic::cond::Inst::target_of(offset, pos)),
		}
	}
}
