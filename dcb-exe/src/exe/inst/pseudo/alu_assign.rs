//! Alu self-assign instructions

// Imports
use crate::exe::inst::{
	basic::{self, alu},
	Register,
};
use std::fmt;

/// Alu assign kind
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Kind {
	/// Immediate
	Imm {
		/// Kind
		kind: alu::imm::Kind,
	},

	/// Register
	Reg {
		/// Kind
		kind: alu::reg::Kind,

		/// Argument
		rhs: Register,
	},
}

impl Kind {
	/// Returns this kind's mnemonic
	#[must_use]
	pub const fn mnemonic(self) -> &'static str {
		match self {
			Self::Imm { kind } => kind.mnemonic(),
			Self::Reg { kind, .. } => kind.mnemonic(),
		}
	}

	/// Returns a displayable with the value of this kind
	#[must_use]
	pub fn value_fmt(self) -> impl fmt::Display {
		/// Display wrapper
		struct FmtValue(Kind);

		impl fmt::Display for FmtValue {
			fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
				match self.0 {
					Kind::Imm { kind } => write!(f, "{}", kind.value_fmt()),
					Kind::Reg { rhs, .. } => write!(f, "{}", rhs),
				}
			}
		}

		FmtValue(self)
	}
}

/// Alu self-assign instructions
///
/// Alias for
/// ```mips
/// [alu] $dst, $dst, $i
/// ```
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[derive(derive_more::Display)]
#[display(fmt = "{} {dst}, {}", "kind.mnemonic()", "kind.value_fmt()")]
pub struct Inst {
	/// Destination and source register
	pub dst: Register,

	/// Kind
	pub kind: Kind,
}

impl Inst {
	/// Decodes this pseudo instruction
	#[must_use]
	pub fn decode(inst: basic::Inst, _bytes: &[u8]) -> Option<(Self, usize)> {
		let inst = match inst {
			basic::Inst::Alu(inst) => match inst {
				alu::Inst::Imm(alu::imm::Inst { dst, lhs, kind }) if dst == lhs => Some(Self {
					dst,
					kind: Kind::Imm { kind },
				}),
				alu::Inst::Reg(alu::reg::Inst { dst, lhs, rhs, kind }) if dst == lhs => Some(Self {
					dst,
					kind: Kind::Reg { kind, rhs },
				}),
				_ => None,
			},
			_ => None,
		};

		inst.map(|inst| (inst, 0))
	}
}
