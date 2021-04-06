//! Shift immediate instructions

// Imports
use crate::inst::{
	basic::{Decodable, Encodable},
	InstFmt, Register,
};
use int_conv::{Truncated, ZeroExtended};

/// Shift immediate instruction kind
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Kind {
	/// Left logical
	LeftLogical,

	/// Right logical
	RightLogical,

	/// Right arithmetic
	RightArithmetic,
}

impl Kind {
	/// Returns this kind's mnemonic
	#[must_use]
	pub const fn mnemonic(self) -> &'static str {
		match self {
			Self::LeftLogical => "sll",
			Self::RightLogical => "srl",
			Self::RightArithmetic => "sra",
		}
	}
}

/// Raw representation
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Raw {
	/// Rt
	pub t: u32,

	/// Rd
	pub d: u32,

	/// Immediate
	pub i: u32,

	/// Function (lower 2 bits)
	pub f: u32,
}

/// Shift immediate instructions
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Inst {
	/// Destination register
	pub dst: Register,

	/// Lhs argument
	pub lhs: Register,

	/// Rhs argument
	pub rhs: u8,

	/// Kind
	pub kind: Kind,
}

impl Decodable for Inst {
	type Raw = Raw;

	fn decode(raw: Self::Raw) -> Option<Self> {
		let kind = match raw.f {
			0x0 => Kind::LeftLogical,
			0x2 => Kind::RightLogical,
			0x3 => Kind::RightArithmetic,
			_ => return None,
		};

		Some(Self {
			dst: Register::new(raw.d)?,
			lhs: Register::new(raw.t)?,
			rhs: raw.i.truncated(),
			kind,
		})
	}
}

impl Encodable for Inst {
	fn encode(&self) -> Self::Raw {
		let f = match self.kind {
			Kind::LeftLogical => 0x0,
			Kind::RightLogical => 0x2,
			Kind::RightArithmetic => 0x3,
		};
		let t = self.lhs.idx();
		let d = self.dst.idx();
		let i = self.rhs.zero_extended();

		Raw { t, d, i, f }
	}
}

impl InstFmt for Inst {
	fn fmt(&self, _pos: crate::Pos, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let Self { dst, lhs, rhs, kind } = self;
		let mnemonic = kind.mnemonic();

		// If `$dst` and `$lhs` are the same, only print one of them
		match dst == lhs {
			true => write!(f, "{mnemonic} {dst}, {rhs:#x}"),
			false => write!(f, "{mnemonic} {dst}, {lhs}, {rhs:#x}"),
		}
	}
}
