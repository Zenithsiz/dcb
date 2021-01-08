//! Jump immediate instructions

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
#[derive(derive_more::Display)]
#[display(fmt = "{} {target}", "kind.mnemonic()")]
pub struct Inst {
	/// Target
	pub target: u32,

	/// Kind
	pub kind: Kind,
}

impl Inst {
	/// Decodes this instruction
	#[must_use]
	pub const fn decode(raw: Raw) -> Option<Self> {
		let kind = match raw.p {
			0 => Kind::Jump,
			1 => Kind::JumpLink,
			_ => return None,
		};

		Some(Self { target: raw.i, kind })
	}

	/// Encodes this instruction
	#[must_use]
	pub const fn encode(self) -> Raw {
		let p = match self.kind {
			Kind::Jump => 0,
			Kind::JumpLink => 1,
		};
		let i = self.target;

		Raw { p, i }
	}
}
