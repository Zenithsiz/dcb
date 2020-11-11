//! Move register instruction

// Imports
use crate::exe::instruction::{
	basic::{
		alu_imm::AluImmKind,
		special::{
			alu_reg::AluRegKind,
			shift::{ShiftImmInst, ShiftRegInst},
			AluRegInst, ShiftInst,
		},
		AluImmInst, InstIter, SpecialInst,
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
	pub fn decode(iter: InstIter<'_, impl Iterator<Item = u32> + Clone>) -> Option<Self> {
		let peeker = iter.peeker();
		let inst = match peeker.next()?? {
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
				kind: AluImmKind::Add(0) | AluImmKind::AddUnsigned(0) | AluImmKind::And(0) | AluImmKind::Or(0) | AluImmKind::Xor(0),
			}) => Self { dst, src },
			_ => return None,
		};

		peeker.apply();
		Some(inst)
	}
}
