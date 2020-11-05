//! Alu register instructions

// Imports
use crate::game::exe::instruction::Register;
use int_conv::{Truncated, ZeroExtended};
use std::{convert::TryFrom, fmt};

/// Alu register instruction kind
///
/// Each variant's value is equal to the lower 4 bits of the opcode
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[derive(num_enum::IntoPrimitive, num_enum::TryFromPrimitive)]
#[repr(u8)]
pub enum AluRegKind {
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
	pub dst: Register,

	/// Lhs argument, `rs`
	pub lhs: Register,

	/// Rhs argument, `rt`
	pub rhs: Register,

	/// Kind
	pub kind: AluRegKind,
}

impl AluRegInst {
	/// Decodes this instruction
	#[must_use]
	pub fn decode(raw: AluRegRaw) -> Option<Self> {
		let kind = AluRegKind::try_from(raw.f.truncated::<u8>()).ok()?;

		Some(Self {
			dst: Register::new(raw.d)?,
			lhs: Register::new(raw.s)?,
			rhs: Register::new(raw.t)?,
			kind,
		})
	}

	/// Encodes this instruction
	#[must_use]
	pub fn encode(self) -> AluRegRaw {
		let d = self.dst.idx();
		let s = self.lhs.idx();
		let t = self.rhs.idx();
		let f = u8::from(self.kind).zero_extended::<u32>();

		AluRegRaw { f, t, d, s }
	}
}

impl fmt::Display for AluRegInst {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let Self { dst, lhs, rhs, kind } = self;

		let mnemonic = match kind {
			AluRegKind::Add => "add",
			AluRegKind::AddUnsigned => "addu",
			AluRegKind::Sub => "sub",
			AluRegKind::SubUnsigned => "subu",
			AluRegKind::And => "and",
			AluRegKind::Or => "or",
			AluRegKind::Xor => "xor",
			AluRegKind::Nor => "nor",
			AluRegKind::SetLessThan => "slt",
			AluRegKind::SetLessThanUnsigned => "sltu",
		};

		write!(f, "{mnemonic} {dst}, {lhs}, {rhs}")
	}
}
