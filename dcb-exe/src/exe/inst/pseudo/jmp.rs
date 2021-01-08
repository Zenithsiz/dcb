//! Jump pseudo instructions

// Imports
use crate::exe::inst::{
	basic::{
		cond::CondKind,
		special::{jmp::JmpKind, JmpInst},
		CondInst, InstIter, SpecialInst,
	},
	BasicInst, Pos, Register,
};

/// Jump / Branch instructions
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[derive(derive_more::Display)]
pub enum JmpPseudoInst {
	/// Jump and link with return address
	/// Alias for `jalr $ra, $dst`
	#[display(fmt = "jalr {target}")]
	JalrRa { target: Register },

	/// Branch if equal to zero
	/// Alias for `beq $arg, $zr, offset`
	#[display(fmt = "beqz {arg}, {offset:#x}")]
	Beqz { arg: Register, offset: Pos },

	/// Branch if different from zero
	/// Alias for `bne $arg, $zr, offset`
	#[display(fmt = "bnez {arg}, {offset:#x}")]
	Bnez { arg: Register, offset: Pos },

	/// Jump relative
	/// Alias for `beq $zr, $zr, offset`
	#[display(fmt = "b {offset:#x}")]
	B { offset: Pos },
}

impl JmpPseudoInst {
	/// Decodes this pseudo instruction
	#[must_use]
	pub fn decode(iter: InstIter<'_, impl Iterator<Item = u32> + Clone>) -> Option<Self> {
		let peeker = iter.peeker();
		let inst = match peeker.next()?? {
			BasicInst::Cond(CondInst { arg, offset, kind }) => match kind {
				CondKind::Equal(Register::Zr) => match arg == Register::Zr {
					true => Self::B { offset },
					false => Self::Beqz { arg, offset },
				},
				CondKind::NotEqual(Register::Zr) => Self::Bnez { arg, offset },
				_ => return None,
			},
			BasicInst::Special(SpecialInst::Jmp(JmpInst { target, kind })) => match kind {
				JmpKind::Link(Register::At) => Self::JalrRa { target },
				_ => return None,
			},
			_ => return None,
		};

		peeker.apply();
		Some(inst)
	}
}
