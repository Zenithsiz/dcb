//! Alu register instructions

// Imports
use crate::game::exe::instruction::Register;
use int_conv::{Truncated, ZeroExtended};
use std::{convert::TryFrom, fmt};

/// Alu register func
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[derive(num_enum::IntoPrimitive, num_enum::TryFromPrimitive)]
#[repr(u8)]
pub enum AluRegFunc {
	/// Add
	Add                 = 0x20,

	/// Add unsigned
	AddUnsigned         = 0x21,

	/// Sub
	Sub                 = 0x22,

	/// Sub unsigned
	SubUnsigned         = 0x23,

	/// And
	And                 = 0x24,

	/// Or
	Or                  = 0x25,

	/// Xor
	Xor                 = 0x26,

	/// Nor
	Nor                 = 0x27,

	/// Set less than
	SetLessThan         = 0x2a,

	/// Set less than unsigned
	SetLessThanUnsigned = 0x2b,
}

/// Raw representation
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct AluRegRaw {
	/// Rs
	pub s: u32,

	/// Rt
	pub t: u32,

	/// Rd
	pub d: u32,

	/// Func
	pub f: u32,
}

/// Alu register instructions
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct AluRegInst {
	/// Destination register, `rd`
	pub dest: Register,

	/// Lhs argument, `rs`
	pub lhs: Register,

	/// Rhs argument, `rt`
	pub rhs: Register,

	/// Function
	pub func: AluRegFunc,
}

impl AluRegInst {
	/// Decodes this instruction
	#[must_use]
	pub fn decode(raw: AluRegRaw) -> Option<Self> {
		let func = AluRegFunc::try_from(raw.f.truncated::<u8>()).ok()?;

		Some(Self {
			dest: Register::new(raw.d)?,
			lhs: Register::new(raw.s)?,
			rhs: Register::new(raw.t)?,
			func,
		})
	}

	/// Encodes this instruction
	#[must_use]
	pub fn encode(self) -> AluRegRaw {
		let d = self.dest.idx();
		let s = self.lhs.idx();
		let t = self.rhs.idx();
		let f = u8::from(self.func).zero_extended::<u32>();

		AluRegRaw { f, t, d, s }
	}
}

impl fmt::Display for AluRegInst {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let Self { dest, lhs, rhs, func } = self;

		let mnemonic = match func {
			AluRegFunc::Add => "add",
			AluRegFunc::AddUnsigned => "addu",
			AluRegFunc::Sub => "sub",
			AluRegFunc::SubUnsigned => "subu",
			AluRegFunc::And => "and",
			AluRegFunc::Or => "or",
			AluRegFunc::Xor => "xor",
			AluRegFunc::Nor => "nor",
			AluRegFunc::SetLessThan => "slt",
			AluRegFunc::SetLessThanUnsigned => "sltu",
		};

		write!(f, "{mnemonic} {dest}, {lhs}, {rhs}")
	}
}
