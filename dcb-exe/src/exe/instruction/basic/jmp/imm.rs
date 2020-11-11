//! Jump immediate instructions

/// Jmp immediate instruction kind
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum JmpImmInstKind {
	/// Jump
	Jump,

	/// Jump and link
	JumpLink,
}

impl JmpImmInstKind {
	/// Returns this kind's mnemonic
	pub fn mnemonic(self) -> &'static str {
		match self {
			JmpImmInstKind::Jump => "j",
			JmpImmInstKind::JumpLink => "jal",
		}
	}
}

/// Raw representation
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct JmpImmInstRaw {
	/// Opcode (lower bit)
	pub p: u32,

	/// Immediate
	pub i: u32,
}

/// Jmp register instructions
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[derive(derive_more::Display)]
#[display(fmt = "{} {target}", "kind.mnemonic()")]
pub struct JmpImmInst {
	/// Target
	pub target: u32,

	/// Kind
	pub kind: JmpImmInstKind,
}

impl JmpImmInst {
	/// Decodes this instruction
	#[must_use]
	pub fn decode(raw: JmpImmInstRaw) -> Option<Self> {
		let kind = match raw.p {
			0 => JmpImmInstKind::Jump,
			1 => JmpImmInstKind::JumpLink,
			_ => return None,
		};

		Some(Self { target: raw.i, kind })
	}

	/// Encodes this instruction
	#[must_use]
	pub fn encode(self) -> JmpImmInstRaw {
		let (p, i) = match self.kind {
			JmpImmInstKind::Jump => 0,
			JmpImmInstKind::JumpLink => 1,
		};
		let i = self.target;

		JmpImmInstRaw { p, i }
	}
}
