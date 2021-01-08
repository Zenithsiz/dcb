//! Nop

// Imports
use crate::exe::inst::{
	basic::{
		special::{
			shift::{reg::ShiftRegFunc, ShiftImmInst},
			ShiftInst,
		},
		InstIter, SpecialInst,
	},
	BasicInst, Register,
};

/// No-op
///
/// Alias for `sll $zr, $zr, 0`.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[derive(derive_more::Display)]
#[display(fmt = "nop ")]
pub struct NopInst {}

impl NopInst {
	/// Decodes this pseudo instruction
	#[must_use]
	pub fn decode(iter: InstIter<'_, impl Iterator<Item = u32> + Clone>) -> Option<Self> {
		let peeker = iter.peeker();
		let inst = match peeker.next()?? {
			BasicInst::AluImm(SpecialInst::Shift(ShiftInst::Imm(ShiftImmInst {
				dst: Register::Zr,
				lhs: Register::Zr,
				rhs: 0,
				func: ShiftRegFunc::LeftLogical,
			}))) => Self,

			_ => return None,
		};

		peeker.apply();
		return inst;
	}
}
