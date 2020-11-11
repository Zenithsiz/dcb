//! Alu self-assign instructions

// Imports
use crate::exe::instruction::{
	basic::{
		alu_imm::AluImmKind,
		special::{alu_reg::AluRegKind, AluRegInst},
		AluImmInst, InstIter, SpecialInst,
	},
	BasicInst, Register,
};
use std::fmt;

/// Alu assign kind
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum AluAssignKind {
	/// Immediate
	Imm(AluImmKind),

	/// Register
	Reg(AluRegKind, Register),
}

impl AluAssignKind {
	/// Returns this kind's mnemonic
	pub fn mnemonic(self) -> &'static str {
		match self {
			AluAssignKind::Imm(kind) => kind.mnemonic(),
			AluAssignKind::Reg(kind, _) => kind.mnemonic(),
		}
	}

	/// Returns a displayable with the value of this kind
	pub fn value_fmt(self) -> impl fmt::Display {
		struct FmtValue(Self);

		impl fmt::Display for FmtValue {
			fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
				match self.0 {
					AluAssignKind::Imm(kind) => write!(f, "{}", kind.value_fmt()),
					AluAssignKind::Reg(kind, reg) => write!(f, "{}", reg),
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
pub struct AluAssignInst {
	/// Destination and source register
	pub dst: Register,

	/// Kind
	pub kind: AluAssignKind,
}

impl AluAssignInst {
	/// Decodes this pseudo instruction
	#[must_use]
	pub fn decode(iter: InstIter<'_, impl Iterator<Item = u32> + Clone>) -> Option<Self> {
		let peeker = iter.peeker();
		let inst = match peeker.next()?? {
			BasicInst::Special(SpecialInst::Alu(AluRegInst { dst, lhs, rhs, kind })) if dst == lhs => Self {
				dst,
				kind: AluAssignKind::Reg(kind, rhs),
			},
			BasicInst::AluImm(AluImmInst { dst, lhs, kind }) if dst == lhs => Self {
				dst,
				kind: AluAssignKind::Imm(kind),
			},
			_ => return None,
		};

		peeker.apply();
		Some(inst)
	}
}
