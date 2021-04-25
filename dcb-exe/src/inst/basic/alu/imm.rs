//! Alu immediate instructions

// Imports
use crate::inst::{
	basic::{Decodable, Encodable, ModifiesReg},
	InstFmt, Register,
};
use dcb_util::SignedHex;
use int_conv::{Signed, Truncated, ZeroExtended};
use std::fmt;

/// Alu immediate instruction kind
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Kind {
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

impl Kind {
	/// Returns this kind's mnemonic
	#[must_use]
	pub const fn mnemonic(self) -> &'static str {
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
	#[must_use]
	pub fn value_fmt(self) -> impl fmt::Display {
		dcb_util::DisplayWrapper::new(move |f| match self {
			// Signed
			Self::Add(rhs) | Self::AddUnsigned(rhs) | Self::SetLessThan(rhs) => write!(f, "{:#}", SignedHex(rhs)),
			// Unsigned
			Self::SetLessThanUnsigned(rhs) | Self::And(rhs) | Self::Or(rhs) | Self::Xor(rhs) => write!(f, "{rhs:#x}"),
		})
	}
}

/// Raw representation
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Raw {
	/// Opcode (lower 3 bits)
	pub p: u32,

	/// Rs
	pub s: u32,

	/// Rt
	pub t: u32,

	/// Immediate
	pub i: u32,
}

/// Alu immediate instructions
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Inst {
	/// Destination register
	pub dst: Register,

	/// Lhs argument
	pub lhs: Register,

	/// Kind
	pub kind: Kind,
}

impl Decodable for Inst {
	type Raw = Raw;

	fn decode(raw: Self::Raw) -> Option<Self> {
		#[rustfmt::skip]
		let kind = match raw.p {
			0x0 => Kind::Add                (raw.i.truncated::<u16>().as_signed()),
			0x1 => Kind::AddUnsigned        (raw.i.truncated::<u16>().as_signed()),
			0x2 => Kind::SetLessThan        (raw.i.truncated::<u16>().as_signed()),
			0x3 => Kind::SetLessThanUnsigned(raw.i.truncated::<u16>()),
			0x4 => Kind::And                (raw.i.truncated::<u16>()),
			0x5 => Kind::Or                 (raw.i.truncated::<u16>()),
			0x6 => Kind::Xor                (raw.i.truncated::<u16>()),
			_ => return None,
		};

		Some(Self {
			dst: Register::new(raw.t)?,
			lhs: Register::new(raw.s)?,
			kind,
		})
	}
}

impl Encodable for Inst {
	fn encode(&self) -> Self::Raw {
		#[rustfmt::skip]
		let (p, i) = match self.kind {
			Kind::Add                (rhs) => (0x0, rhs.as_unsigned().zero_extended::<u32>()),
			Kind::AddUnsigned        (rhs) => (0x1, rhs.as_unsigned().zero_extended::<u32>()),
			Kind::SetLessThan        (rhs) => (0x2, rhs.as_unsigned().zero_extended::<u32>()),
			Kind::SetLessThanUnsigned(rhs) => (0x3, rhs.as_unsigned().zero_extended::<u32>()),
			Kind::And                (rhs) => (0x4, rhs.as_unsigned().zero_extended::<u32>()),
			Kind::Or                 (rhs) => (0x5, rhs.as_unsigned().zero_extended::<u32>()),
			Kind::Xor                (rhs) => (0x6, rhs.as_unsigned().zero_extended::<u32>()),
		};
		let s = self.lhs.idx();
		let t = self.dst.idx();

		Raw { p, s, t, i }
	}
}

impl InstFmt for Inst {
	fn fmt(&self, _pos: crate::Pos, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let Self { dst, lhs, kind } = self;
		let mnemonic = kind.mnemonic();
		let value = kind.value_fmt();

		// If `$dst` and `$lhs` are the same, only print one of them
		match dst == lhs {
			true => write!(f, "{mnemonic} {dst}, {value}"),
			false => write!(f, "{mnemonic} {dst}, {lhs}, {value}"),
		}
	}
}

impl ModifiesReg for Inst {
	fn modifies_reg(&self, reg: Register) -> bool {
		self.dst == reg
	}
}
