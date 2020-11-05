//! Alu immediate instructions

// Imports
use crate::{game::exe::instruction::Register, util::SignedHex};
use int_conv::{Signed, Truncated, ZeroExtended};
use std::{convert::TryFrom, fmt};

/// Alu immediate instruction kind
///
/// Each variant's value is equal to the lower 3 bits of the opcode
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[derive(num_enum::IntoPrimitive, num_enum::TryFromPrimitive)]
#[repr(u8)]
pub enum AluImmKind {
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
	pub dst: Register,

	/// Lhs argument, `rs`
	pub lhs: Register,

	/// Rhs argument, immediate
	pub rhs: u32,

	/// Opcode
	pub kind: AluImmKind,
}

impl AluImmInst {
	/// Decodes this instruction
	#[must_use]
	pub fn decode(raw: AluImmRaw) -> Option<Self> {
		let kind = AluImmKind::try_from(raw.p.truncated::<u8>()).ok()?;

		Some(Self {
			dst: Register::new(raw.t)?,
			lhs: Register::new(raw.s)?,
			rhs: raw.i,
			kind,
		})
	}

	/// Encodes this instruction
	#[must_use]
	pub fn encode(self) -> AluImmRaw {
		let p = u8::from(self.kind).zero_extended::<u32>();
		let s = self.lhs.idx();
		let t = self.dst.idx();
		let i = self.rhs;

		AluImmRaw { p, s, t, i }
	}
}

impl fmt::Display for AluImmInst {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let Self { kind, dst, lhs, rhs } = self;

		#[rustfmt::skip]
		match kind {
			AluImmKind::Add                 => write!(f, "addi {dst}, {lhs}, {:#x}", SignedHex(rhs.as_signed())),
			AluImmKind::AddUnsigned         => write!(f, "addiu {dst}, {lhs}, {:#x}", SignedHex(rhs.as_signed())),
			AluImmKind::SetLessThan         => write!(f, "slti {dst}, {lhs}, {:#x}", SignedHex(rhs.as_signed())),
			AluImmKind::SetLessThanUnsigned => write!(f, "sltiu {dst}, {lhs}, {rhs:#x}"),
			AluImmKind::And                 => write!(f, "andi {dst}, {lhs}, {rhs:#x}"),
			AluImmKind::Or                  => write!(f, "ori {dst}, {lhs}, {rhs:#x}"),
			AluImmKind::Xor                 => write!(f, "xori {dst}, {lhs}, {rhs:#x}"),
		}
	}
}
