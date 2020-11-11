//! Jump register instructions

// Imports
use crate::exe::instruction::Register;
use std::fmt;

/// Jmp register instruction kind
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum JmpRegInstKind {
	/// Jump
	Jump,

	/// Jump and link
	JumpLink(Register),
}

impl JmpRegInstKind {
	/// Returns this kind's mnemonic
	pub fn mnemonic(self) -> &'static str {
		match self {
			JmpRegInstKind::Jump => "j",
			JmpRegInstKind::JumpLink(_) => "jal",
		}
	}
}

/// Raw representation
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct JmpRegInstRaw {
	/// Rs
	s: u32,

	/// Rd
	d: u32,

	/// Func (lower bit)
	f: u32,
}

/// Jmp register instructions
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct JmpRegInst {
	/// Target
	pub target: Register,

	/// Kind
	pub kind: JmpRegInstKind,
}

impl JmpRegInst {
	/// Decodes this instruction
	#[must_use]
	pub fn decode(raw: JmpRegInstRaw) -> Option<Self> {
		let kind = match raw.f {
			0 => JmpRegInstKind::Jump,
			1 => JmpRegInstKind::JumpLink(Register::new(raw.d)?),
			_ => return None,
		};
		let target = Register::new(raw.s)?;

		Some(Self { target, kind })
	}

	/// Encodes this instruction
	#[must_use]
	pub fn encode(self) -> JmpRegInstRaw {
		let (f, d) = match self.kind {
			JmpRegInstKind::Jump => (0, 0),
			JmpRegInstKind::JumpLink(reg) => (1, reg.idx()),
		};
		let s = self.target.idx();

		JmpRegInstRaw { s, d, f }
	}
}

impl fmt::Display for JmpRegInst {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let mnemonic = self.kind.mnemonic();
		let target = self.target;

		match self.kind {
			JmpRegInstKind::Jump => write!(f, "{mnemonic} {target}"),
			JmpRegInstKind::JumpLink(reg) => write!(f, "{mnemonic} {target}, {reg}"),
		}
	}
}
