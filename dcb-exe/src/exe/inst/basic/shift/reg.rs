//! Shift register instructions

// Imports
use crate::exe::inst::{
	basic::{Decodable, Encodable},
	InstFmt, Register,
};

/// Shift register instruction kind
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
			Self::LeftLogical => "sllv",
			Self::RightLogical => "srlv",
			Self::RightArithmetic => "srav",
		}
	}
}

/// Raw representation
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Raw {
	/// Rs
	pub s: u32,

	/// Rt
	pub t: u32,

	/// Rd
	pub d: u32,

	/// Func (lower 4 bits)
	pub f: u32,
}

/// Shift register instructions
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Inst {
	/// Destination register
	pub dst: Register,

	/// Lhs argument
	pub lhs: Register,

	/// Rhs argument
	pub rhs: Register,

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
			rhs: Register::new(raw.s)?,
			kind,
		})
	}
}
impl Encodable for Inst {
	fn encode(&self) -> Raw {
		let f = match self.kind {
			Kind::LeftLogical => 0x0,
			Kind::RightLogical => 0x2,
			Kind::RightArithmetic => 0x3,
		};

		let d = self.dst.idx();
		let t = self.lhs.idx();
		let s = self.rhs.idx();

		Raw { s, t, d, f }
	}
}

impl InstFmt for Inst {
	fn mnemonic(&self) -> &'static str {
		self.kind.mnemonic()
	}

	fn fmt(&self, _pos: crate::Pos, _bytes: &[u8], f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let Self { dst, lhs, rhs, kind } = self;
		let mnemonic = kind.mnemonic();

		write!(f, "{mnemonic} {dst}, {lhs}, {rhs}")
	}
}
