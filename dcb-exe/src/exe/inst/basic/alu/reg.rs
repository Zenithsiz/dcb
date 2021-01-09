//! Alu register instructions

// Imports
use crate::exe::inst::{
	basic::{Decodable, Encodable},
	InstFmt, Register,
};

/// Alu register instruction kind
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Kind {
	/// Add signed with overflow trap
	Add,

	/// Add signed without overflow trap
	AddUnsigned,

	/// Sub signed with overflow trap
	Sub,

	/// Sub signed without overflow trap
	SubUnsigned,

	/// Bit and
	And,

	/// Bit or
	Or,

	/// Bit xor
	Xor,

	/// Bit nor
	Nor,

	/// Set on less than signed
	SetLessThan,

	/// Set on less than unsigned
	SetLessThanUnsigned,
}

impl Kind {
	/// Returns this kind's mnemonic
	#[must_use]
	pub const fn mnemonic(self) -> &'static str {
		match self {
			Self::Add => "add",
			Self::AddUnsigned => "addu",
			Self::Sub => "sub",
			Self::SubUnsigned => "subu",
			Self::And => "and",
			Self::Or => "or",
			Self::Xor => "xor",
			Self::Nor => "nor",
			Self::SetLessThan => "slt",
			Self::SetLessThanUnsigned => "sltu",
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

/// Alu register instructions
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
			0x0 => Kind::Add,
			0x1 => Kind::AddUnsigned,
			0x2 => Kind::Sub,
			0x3 => Kind::SubUnsigned,
			0x4 => Kind::And,
			0x5 => Kind::Or,
			0x6 => Kind::Xor,
			0x7 => Kind::Nor,
			0xa => Kind::SetLessThan,
			0xb => Kind::SetLessThanUnsigned,
			_ => return None,
		};

		Some(Self {
			dst: Register::new(raw.d)?,
			lhs: Register::new(raw.s)?,
			rhs: Register::new(raw.t)?,
			kind,
		})
	}
}
impl Encodable for Inst {
	fn encode(&self) -> Self::Raw {
		let f = match self.kind {
			Kind::Add => 0x0,
			Kind::AddUnsigned => 0x1,
			Kind::Sub => 0x2,
			Kind::SubUnsigned => 0x3,
			Kind::And => 0x4,
			Kind::Or => 0x5,
			Kind::Xor => 0x6,
			Kind::Nor => 0x7,
			Kind::SetLessThan => 0xa,
			Kind::SetLessThanUnsigned => 0xb,
		};

		let d = self.dst.idx();
		let s = self.lhs.idx();
		let t = self.rhs.idx();

		Raw { f, t, d, s }
	}
}

impl InstFmt for Inst {
	fn mnemonic(&self) -> &'static str {
		self.kind.mnemonic()
	}

	fn fmt(&self, _pos: crate::Pos, _bytes: &[u8], f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let Self { dst, lhs, rhs, kind } = self;

		write!(f, "{} {dst}, {lhs}, {rhs}", kind.mnemonic())
	}
}
