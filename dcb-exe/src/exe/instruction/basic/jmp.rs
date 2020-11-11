//! Alu register instructions

// Imports
use std::fmt;

/// Alu register func (bottom bit)
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum JmpKind {
	/// Jump
	Jump,

	/// Jump and link
	Link,
}

/// Raw representation
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct JmpRaw {
	/// Opcode (bottom bit)
	pub p: u32,

	/// Immediate
	pub i: u32,
}

/// Alu register instructions
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct JmpInst {
	/// Target
	pub target: u32,

	/// Kind
	pub kind: JmpKind,
}

impl JmpInst {
	/// Decodes this instruction
	#[must_use]
	pub fn decode(raw: JmpRaw) -> Self {
		let kind = match raw.p {
			0 => JmpKind::Jump,
			1 => JmpKind::Link,
			_ => unreachable!("Received invalid bit in opcode."),
		};

		Self { target: raw.i, kind }
	}

	/// Encodes this instruction
	#[must_use]
	pub const fn encode(self) -> JmpRaw {
		let p = match self.kind {
			JmpKind::Jump => 0,
			JmpKind::Link => 1,
		};

		JmpRaw { p, i: self.target }
	}
}

// TODO: Format with `pc` / `label`.

impl fmt::Display for JmpInst {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let Self { target, kind } = self;

		let mnemonic = match kind {
			JmpKind::Jump => "j",
			JmpKind::Link => "jal",
		};

		write!(f, "{mnemonic} {target:#x}")
	}
}
