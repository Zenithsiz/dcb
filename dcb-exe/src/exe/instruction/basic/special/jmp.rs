//! Jumps

// Imports
use crate::exe::instruction::Register;
use std::fmt;

/// Jump kind
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum JmpKind {
	/// Simple
	Simple,

	/// With link
	Link(Register),
}

/// Raw representation
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct JmpRaw {
	/// Rs
	pub s: u32,

	/// Rd
	pub d: u32,

	/// Func
	pub f: u32,
}

/// Jump instructions
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct JmpInst {
	/// Target register, `rs`
	pub target: Register,

	/// Jump kind, `rs`if `jalr`.
	pub kind: JmpKind,
}

impl JmpInst {
	/// Decodes this instruction
	#[must_use]
	pub fn decode(raw: JmpRaw) -> Option<Self> {
		let kind = match raw.f {
			0x8 => JmpKind::Simple,
			0x9 => JmpKind::Link(Register::new(raw.d)?),
			_ => return None,
		};

		Some(Self {
			target: Register::new(raw.s)?,
			kind,
		})
	}

	/// Encodes this instruction
	#[must_use]
	pub fn encode(self) -> JmpRaw {
		let s = self.target.idx();
		let (d, f) = match self.kind {
			JmpKind::Simple => (0, 0x8),
			JmpKind::Link(d) => (d.idx(), 0x9),
		};

		JmpRaw { s, d, f }
	}
}

impl fmt::Display for JmpInst {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let Self { target, kind } = self;

		match kind {
			JmpKind::Simple => write!(f, "jr {target}"),
			JmpKind::Link(link) => write!(f, "jalr {link}, {target}"),
		}
	}
}
