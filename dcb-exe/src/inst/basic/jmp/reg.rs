//! Jump register instructions

// Imports
use crate::inst::{
	basic::{Decodable, Encodable, ModifiesReg},
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

/// Jmp register instructions
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Inst {
	/// Target
	pub target: Register,

	/// Kind
	pub kind: Kind,
}

impl Decodable for Inst {
	type Raw = u32;

	#[bitmatch::bitmatch]
	fn decode(raw: Self::Raw) -> Option<Self> {
		let [s, d, f] = #[bitmatch]
		match raw {
			"000000_sssss_?????_ddddd_?????_00100f" => [s, d, f],
			_ => return None,
		};

		let kind = match f {
			0 => Kind::Jump,
			1 => Kind::JumpLink(Register::new(d)?),
			_ => return None,
		};
		let target = Register::new(s)?;

		Some(Self { target, kind })
	}
}

impl Encodable for Inst {
	#[bitmatch::bitmatch]
	fn encode(&self) -> Self::Raw {
		let (f, d): (u32, u32) = match self.kind {
			Kind::Jump => (0, 0),
			Kind::JumpLink(reg) => (1, reg.idx()),
		};
		let s = self.target.idx();

		bitpack!("000000_sssss_?????_ddddd_?????_00100f")
	}
}

impl InstFmt for Inst {
	fn fmt(&self, _pos: crate::Pos, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let Self { target, kind } = self;
		let mnemonic = kind.mnemonic();

		match kind {
			Kind::Jump => write!(f, "{mnemonic} {target}"),
			Kind::JumpLink(reg) => match reg {
				// If using `$ra`, don't output it.
				Register::Ra => write!(f, "{mnemonic} {target}"),
				reg => write!(f, "{mnemonic} {target}, {reg}"),
			},
		}
	}
}

impl ModifiesReg for Inst {
	fn modifies_reg(&self, _reg: Register) -> bool {
		false
	}
}
