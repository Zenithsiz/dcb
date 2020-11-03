//! Alu immediate instructions

// Imports
use crate::{game::exe::instruction::Register, util::SignedHex};
use int_conv::{Signed, Truncated, ZeroExtended};
use std::{convert::TryFrom, fmt};

/// Alu register opcode (lower 3 bits)
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[derive(num_enum::IntoPrimitive, num_enum::TryFromPrimitive)]
#[repr(u8)]
pub enum AluImmOp {
	/// Add
	Add                 = 0x0,

	/// Add unsigned
	AddUnsigned         = 0x1,

	/// Set less than
	SetLessThan         = 0x2,

	/// Set less than unsigned
	SetLessThanUnsigned = 0x3,

	/// And
	And                 = 0x4,

	/// Or
	Or                  = 0x5,

	/// Xor
	Xor                 = 0x6,
}

/// Raw representation
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct AluImmRaw {
	/// Opcode (lower 3 bits)
	pub p: u32,

	/// Rs
	pub s: u32,

	/// Rt
	pub t: u32,

	/// Immediate
	pub i: u32,
}

/// Alu register instructions
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct AluImmInst {
	/// Destination register, `rd`
	pub dest: Register,

	/// Lhs argument, `rs`
	pub lhs: Register,

	/// Rhs argument, immediate
	pub rhs: u32,

	/// Opcode
	pub op: AluImmOp,
}

impl AluImmInst {
	/// Decodes this instruction
	#[must_use]
	pub fn decode(raw: AluImmRaw) -> Option<Self> {
		let op = AluImmOp::try_from(raw.p.truncated::<u8>()).ok()?;

		Some(Self {
			dest: Register::new(raw.t)?,
			lhs: Register::new(raw.s)?,
			rhs: raw.i,
			op,
		})
	}

	/// Encodes this instruction
	#[must_use]
	pub fn encode(self) -> AluImmRaw {
		let p = u8::from(self.op).zero_extended::<u32>();
		let s = self.lhs.idx();
		let t = self.dest.idx();
		let i = self.rhs;

		AluImmRaw { p, s, t, i }
	}
}

impl fmt::Display for AluImmInst {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let Self { op, dest, lhs, rhs } = self;

		#[rustfmt::skip]
		match op {
			AluImmOp::Add                 => write!(f, "addi {dest}, {lhs}, {:#x}", SignedHex(rhs.as_signed())),
			AluImmOp::AddUnsigned         => write!(f, "addiu {dest}, {lhs}, {:#x}", SignedHex(rhs.as_signed())),
			AluImmOp::SetLessThan         => write!(f, "slti {dest}, {lhs}, {:#x}", SignedHex(rhs.as_signed())),
			AluImmOp::SetLessThanUnsigned => write!(f, "sltiu {dest}, {lhs}, {rhs:#x}"),
			AluImmOp::And                 => write!(f, "andi {dest}, {lhs}, {rhs:#x}"),
			AluImmOp::Or                  => write!(f, "ori {dest}, {lhs}, {rhs:#x}"),
			AluImmOp::Xor                 => write!(f, "xori {dest}, {lhs}, {rhs:#x}"),
		}
	}
}
