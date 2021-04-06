//! Jump immediate instructions

// Imports
use crate::{
	inst::{
		basic::{Decodable, Encodable},
		InstTarget, InstTargetFmt,
	},
	Pos,
};

/// Jmp immediate instruction kind
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Kind {
	/// Jump
	Jump,

	/// Jump and link
	JumpLink,
}

impl Kind {
	/// Returns this kind's mnemonic
	#[must_use]
	pub const fn mnemonic(self) -> &'static str {
		match self {
			Self::Jump => "j",
			Self::JumpLink => "jal",
		}
	}
}

/// Raw representation
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Raw {
	/// Opcode (lower bit)
	pub p: u32,

	/// Immediate
	pub i: u32,
}

/// Jmp register instructions
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Inst {
	/// Immediate
	pub imm: u32,

	/// Kind
	pub kind: Kind,
}

impl Inst {
	/// Returns the target using an immediate
	#[must_use]
	pub fn target_of(imm: u32, pos: Pos) -> Pos {
		(pos & 0xf0000000) + imm * 4
	}
}

impl Decodable for Inst {
	type Raw = Raw;

	fn decode(raw: Self::Raw) -> Option<Self> {
		let kind = match raw.p {
			0 => Kind::Jump,
			1 => Kind::JumpLink,
			_ => return None,
		};

		Some(Self { imm: raw.i, kind })
	}
}

impl Encodable for Inst {
	fn encode(&self) -> Self::Raw {
		let p = match self.kind {
			Kind::Jump => 0,
			Kind::JumpLink => 1,
		};
		let i = self.imm;

		Raw { p, i }
	}
}

impl InstTarget for Inst {
	fn target(&self, pos: Pos) -> Pos {
		Self::target_of(self.imm, pos)
	}
}

impl InstTargetFmt for Inst {
	fn fmt(&self, _pos: Pos, target: impl std::fmt::Display, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let mnemonic = self.kind.mnemonic();

		write!(f, "{mnemonic} {target}")
	}
}
