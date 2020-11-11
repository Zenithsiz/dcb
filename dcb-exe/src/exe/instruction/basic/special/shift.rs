//! Shift instructions

// Modules
pub mod imm;
pub mod reg;

// Exports
pub use imm::{ShiftImmInst, ShiftImmRaw};
pub use reg::{ShiftRegInst, ShiftRegRaw};

/// Raw representation
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct ShiftRaw {
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

impl From<ShiftRaw> for ShiftImmRaw {
	fn from(ShiftRaw { t, d, i, f, .. }: ShiftRaw) -> Self {
		Self { t, d, i, f }
	}
}

impl From<ShiftImmRaw> for ShiftRaw {
	fn from(ShiftImmRaw { t, d, i, f }: ShiftImmRaw) -> Self {
		Self { t, d, s: 0, i, f }
	}
}

impl From<ShiftRaw> for ShiftRegRaw {
	fn from(ShiftRaw { t, d, s, f, .. }: ShiftRaw) -> Self {
		Self { t, d, s, f }
	}
}

impl From<ShiftRegRaw> for ShiftRaw {
	fn from(ShiftRegRaw { t, d, s, f }: ShiftRegRaw) -> Self {
		Self { t, d, s, i: 0, f }
	}
}

/// Shift instructions
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[derive(derive_more::Display)]
pub enum ShiftInst {
	/// Register
	Reg(ShiftRegInst),

	/// Immediate
	Imm(ShiftImmInst),
}

impl ShiftInst {
	/// Decodes this instruction
	#[must_use]
	pub fn decode(raw: ShiftRaw) -> Option<Self> {
		Some(match raw.f {
			0x0..0x4 => Self::Imm(ShiftImmInst::decode(raw.into())?),
			0x4..0x8 => Self::Reg(ShiftRegInst::decode(raw.into())?),
			_ => return None,
		})
	}

	/// Encodes this instruction
	#[must_use]
	pub fn encode(self) -> ShiftRaw {
		match self {
			Self::Reg(inst) => inst.encode().into(),
			Self::Imm(inst) => inst.encode().into(),
		}
	}
}
