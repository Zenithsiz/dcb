//! ALU instructions

// Modules
pub mod alu_reg;
pub mod jmp;
pub mod mult;
pub mod shift;
pub mod sys;

// Exports
pub use alu_reg::{AluRegInst, AluRegRaw};
pub use jmp::{JmpInst, JmpRaw};
pub use mult::{MultInst, MultRaw};
pub use shift::{ShiftInst, ShiftRaw};
pub use sys::{SysInst, SysRaw};

/// Raw representation
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct SpecialRaw {
	/// Rs
	pub s: u32,

	/// Rt
	pub t: u32,

	/// Rd
	pub d: u32,

	/// Immediate
	pub i: u32,

	/// Func
	pub f: u32,
}

impl From<SpecialRaw> for ShiftRaw {
	fn from(SpecialRaw { t, d, s, i, f }: SpecialRaw) -> Self {
		Self { t, d, s, i, f }
	}
}

impl From<ShiftRaw> for SpecialRaw {
	fn from(ShiftRaw { t, d, s, i, f }: ShiftRaw) -> Self {
		Self { t, d, s, i, f }
	}
}

impl From<SpecialRaw> for JmpRaw {
	fn from(SpecialRaw { d, s, f, .. }: SpecialRaw) -> Self {
		Self { d, s, f }
	}
}

impl From<JmpRaw> for SpecialRaw {
	fn from(JmpRaw { d, s, f }: JmpRaw) -> Self {
		Self { t: 0, d, s, i: 0, f }
	}
}

impl From<SpecialRaw> for SysRaw {
	fn from(SpecialRaw { t, d, s, i, f }: SpecialRaw) -> Self {
		Self { t, d, s, i, f }
	}
}

impl From<SysRaw> for SpecialRaw {
	fn from(SysRaw { t, d, s, i, f }: SysRaw) -> Self {
		Self { t, d, s, i, f }
	}
}

impl From<SpecialRaw> for MultRaw {
	fn from(SpecialRaw { t, d, s, f, .. }: SpecialRaw) -> Self {
		Self { t, d, s, f }
	}
}

impl From<MultRaw> for SpecialRaw {
	fn from(MultRaw { t, d, s, f }: MultRaw) -> Self {
		Self { t, d, s, i: 0, f }
	}
}

impl From<SpecialRaw> for AluRegRaw {
	fn from(SpecialRaw { t, d, s, f, .. }: SpecialRaw) -> Self {
		Self { t, d, s, f }
	}
}

impl From<AluRegRaw> for SpecialRaw {
	fn from(AluRegRaw { t, d, s, f }: AluRegRaw) -> Self {
		Self { t, d, s, i: 0, f }
	}
}

/// Special instructions for `opcode: 0`
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[derive(derive_more::Display)]
pub enum SpecialInst {
	/// Shift instruction
	Shift(ShiftInst),

	/// Jump
	Jmp(JmpInst),

	/// Sys
	Sys(SysInst),

	/// Mult
	Mult(MultInst),

	/// Alu
	Alu(AluRegInst),
}

impl SpecialInst {
	/// Decodes this instruction
	#[must_use]
	pub fn decode(raw: SpecialRaw) -> Option<Self> {
		Some(match raw.f {
			0x00..0x08 => Self::Shift(ShiftInst::decode(raw.into())?),
			0x08..0x0c => Self::Jmp(JmpInst::decode(raw.into())?),
			0x0c..0x10 => Self::Sys(SysInst::decode(raw.into())?),
			0x10..0x20 => Self::Mult(MultInst::decode(raw.into())?),
			0x20..0x30 => Self::Alu(AluRegInst::decode(raw.into())?),
			0x30..0x40 => return None,

			_ => unreachable!("Func was larger than 6 bits."),
		})
	}

	/// Encodes this instruction
	#[must_use]
	pub fn encode(self) -> SpecialRaw {
		match self {
			Self::Shift(inst) => inst.encode().into(),
			Self::Jmp(inst) => inst.encode().into(),
			Self::Sys(inst) => inst.encode().into(),
			Self::Mult(inst) => inst.encode().into(),
			Self::Alu(inst) => inst.encode().into(),
		}
	}
}
