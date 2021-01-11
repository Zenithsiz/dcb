//! Jump register instructions

// Imports
use crate::exe::inst::{
	basic::{Decodable, Encodable},
	InstFmt, Register,
};

/// Jmp register instruction kind
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Kind {
	/// Jump
	Jump,

	/// Jump and link
	JumpLink(Register),
}

impl Kind {
	/// Returns this kind's mnemonic
	#[must_use]
	pub const fn mnemonic(self) -> &'static str {
		match self {
			Self::Jump => "jr",
			Self::JumpLink(_) => "jalr",
		}
	}
}

/// Raw representation
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Raw {
	/// Rs
	pub s: u32,

	/// Rd
	pub d: u32,

	/// Func (lower bit)
	pub f: u32,
}

/// Jmp register instructions
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Inst {
	/// Target
	pub target: Register,

	/// Kind
	pub kind: Kind,
}

impl Decodable for Inst {
	type Raw = Raw;

	fn decode(raw: Self::Raw) -> Option<Self> {
		let kind = match raw.f {
			0 => Kind::Jump,
			1 => Kind::JumpLink(Register::new(raw.d)?),
			_ => return None,
		};
		let target = Register::new(raw.s)?;

		Some(Self { target, kind })
	}
}

impl Encodable for Inst {
	fn encode(&self) -> Self::Raw {
		let (f, d) = match self.kind {
			Kind::Jump => (0, 0),
			Kind::JumpLink(reg) => (1, reg.idx()),
		};
		let s = self.target.idx();

		Raw { s, d, f }
	}
}

impl InstFmt for Inst {
	fn fmt(&self, _pos: crate::Pos, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let Self { target, kind } = self;
		let mnemonic = kind.mnemonic();

		match kind {
			Kind::Jump => write!(f, "{mnemonic} {target}"),
			Kind::JumpLink(reg) => write!(f, "{mnemonic} {target}, {reg}"),
		}
	}
}
