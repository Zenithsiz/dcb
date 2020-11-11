//! Alu immediate instructions

// Imports
use crate::exe::instruction::Register;
use dcb_util::SignedHex;
use int_conv::{Signed, Truncated, ZeroExtended};
use std::fmt;

/// Alu immediate instruction kind
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum AluImmInstKind {
	/// Add signed with overflow trap
	Add(i16),

	/// Add signed without overflow trap
	AddUnsigned(i16),

	/// Set on less than signed
	SetLessThan(i16),

	/// Set on less than unsigned
	SetLessThanUnsigned(u16),

	/// Bit and
	And(u16),

	/// Bit or
	Or(u16),

	/// Bit xor
	Xor(u16),
}

impl AluImmInstKind {
	/// Returns this kind's mnemonic
	pub fn mnemonic(self) -> &'static str {
		match self {
			Self::Add(_) => "addi",
			Self::AddUnsigned(_) => "addiu",
			Self::SetLessThan(_) => "slti",
			Self::SetLessThanUnsigned(_) => "sltiu",
			Self::And(_) => "andi",
			Self::Or(_) => "ori",
			Self::Xor(_) => "xori",
		}
	}

	/// Returns a displayable with the value of this kind
	pub fn value_fmt(self) -> impl fmt::Display {
		struct FmtValue(Self);

		impl fmt::Display for FmtValue {
			fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
				use AluImmInstKind::*;
				match self.0 {
					// Signed
					Add(rhs) | AddUnsigned(rhs) | SetLessThan(rhs) => write!(f, "{:#x}", SignedHex(rhs)),
					// Unsigned
					SetLessThanUnsigned(rhs) | And(rhs) | Or(rhs) | Xor(rhs) => write!(f, "{rhs:#x}"),
				}
			}
		}

		FmtValue(self)
	}
}

/// Raw representation
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct AluImmInstRaw {
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
#[derive(derive_more::Display)]
#[display(fmt = "{} {dst}, {lhs}, {}", "kind.mnemonic()", "kind.value_fmt()")]
pub struct AluImmInst {
	/// Destination register
	pub dst: Register,

	/// Lhs argument
	pub lhs: Register,

	/// Kind
	pub kind: AluImmInstKind,
}

impl AluImmInst {
	/// Decodes this instruction
	#[must_use]
	pub fn decode(raw: AluImmInstRaw) -> Option<Self> {
		#[rustfmt::skip]
		let kind = match raw.p {
			0x0 => AluImmInstKind::Add                (raw.i.truncated::<u16>().as_signed()),
			0x1 => AluImmInstKind::AddUnsigned        (raw.i.truncated::<u16>().as_signed()),
			0x2 => AluImmInstKind::SetLessThan        (raw.i.truncated::<u16>().as_signed()),
			0x3 => AluImmInstKind::SetLessThanUnsigned(raw.i.truncated::<u16>()),
			0x4 => AluImmInstKind::And                (raw.i.truncated::<u16>()),
			0x5 => AluImmInstKind::Or                 (raw.i.truncated::<u16>()),
			0x6 => AluImmInstKind::Xor                (raw.i.truncated::<u16>()),
			_ => return None,
		};

		Some(Self {
			dst: Register::new(raw.t)?,
			lhs: Register::new(raw.s)?,
			kind,
		})
	}

	/// Encodes this instruction
	#[must_use]
	pub fn encode(self) -> AluImmInstRaw {
		#[rustfmt::skip]
		let (p, i) = match self.kind {
			AluImmInstKind::Add                (rhs) => (0x0, rhs.zero_extended::<u32>()),
			AluImmInstKind::AddUnsigned        (rhs) => (0x1, rhs.zero_extended::<u32>()),
			AluImmInstKind::SetLessThan        (rhs) => (0x2, rhs.zero_extended::<u32>()),
			AluImmInstKind::SetLessThanUnsigned(rhs) => (0x3, rhs.zero_extended::<u32>()),
			AluImmInstKind::And                (rhs) => (0x4, rhs.zero_extended::<u32>()),
			AluImmInstKind::Or                 (rhs) => (0x5, rhs.zero_extended::<u32>()),
			AluImmInstKind::Xor                (rhs) => (0x6, rhs.zero_extended::<u32>()),
		};
		let s = self.lhs.idx();
		let t = self.dst.idx();

		AluImmInstRaw { p, s, t, i }
	}
}
