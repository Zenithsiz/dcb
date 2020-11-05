//! Move register instruction

// Imports
use crate::game::exe::instruction::{
	basic::{
		alu_imm::AluImmKind,
		special::{
			alu_reg::AluRegKind,
			shift::{ShiftImmInst, ShiftRegInst},
			AluRegInst, ShiftInst,
		},
		AluImmInst, SpecialInst,
	},
	BasicInst, Register,
};

/// Move register instruction
///
/// Alias for
/// ```mips
/// Alias for `{add|addu|sub|subu|and|or|xor|sllv|srlv|srav} $dst, $src, $zr` or
/// `{addi|addiu|andi|ori|xori|sll|srl|sra} $dst, $src, 0`
/// ```
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[derive(derive_more::Display)]
#[display(fmt = "move {dst}, {src}")]
pub struct MoveRegPseudoInst {
	/// Destination register
	pub dst: Register,

	/// Source register
	pub src: Register,
}

impl MoveRegPseudoInst {
	/// Decodes this pseudo instruction
	#[must_use]
	pub const fn decode(inst: BasicInst) -> Option<Self> {
		match inst {
			BasicInst::Special(
				SpecialInst::Shift(
					ShiftInst::Imm(ShiftImmInst { dst, lhs: src, rhs: 0, .. }) |
					ShiftInst::Reg(ShiftRegInst {
						dst,
						lhs: src,
						rhs: Register::Zr,
						..
					}),
				) |
				SpecialInst::Alu(AluRegInst {
					dst,
					lhs: src,
					rhs: Register::Zr,
					kind:
						AluRegKind::Add |
						AluRegKind::AddUnsigned |
						AluRegKind::Sub |
						AluRegKind::SubUnsigned |
						AluRegKind::And |
						AluRegKind::Or |
						AluRegKind::Xor |
						AluRegKind::Nor,
				}),
			) |
			BasicInst::AluImm(AluImmInst {
				dst,
				lhs: src,
				rhs: 0,
				kind: AluImmKind::Add | AluImmKind::AddUnsigned | AluImmKind::And | AluImmKind::Or | AluImmKind::Xor,
			}) => Some(Self { dst, src }),
			_ => None,
		}
	}
}
