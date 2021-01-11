//! Alu self-assign instructions

// Imports
use super::Decodable;
use crate::exe::inst::{
	basic::{self, alu},
	InstFmt, InstSize, Register,
};
use std::{convert::TryInto, fmt};

/// Alu assign kind
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Kind {
	/// Immediate
	Imm {
		/// Kind
		kind: alu::imm::Kind,
	},

	/// Register
	Reg {
		/// Kind
		kind: alu::reg::Kind,

		/// Argument
		rhs: Register,
	},
}

impl Kind {
	/// Returns this kind's mnemonic
	#[must_use]
	pub const fn mnemonic(self) -> &'static str {
		match self {
			Self::Imm { kind } => kind.mnemonic(),
			Self::Reg { kind, .. } => kind.mnemonic(),
		}
	}

	/// Returns a displayable with the value of this kind
	#[must_use]
	pub fn value_fmt(self) -> impl fmt::Display {
		dcb_util::DisplayWrapper::new(move |f| match self {
			Self::Imm { kind } => write!(f, "{}", kind.value_fmt()),
			Self::Reg { rhs, .. } => write!(f, "{}", rhs),
		})
	}
}

/// Alu self-assign instructions
///
/// Alias for
/// ```mips
/// [alu] $dst, $dst, $i
/// ```
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Inst {
	/// Destination and source register
	pub dst: Register,

	/// Kind
	pub kind: Kind,
}

impl Decodable for Inst {
	fn decode(mut insts: impl Iterator<Item = basic::Inst> + Clone) -> Option<Self> {
		match insts.next()?.try_into().ok()? {
			alu::Inst::Imm(alu::imm::Inst { dst, lhs, kind }) if dst == lhs => Some(Self {
				dst,
				kind: Kind::Imm { kind },
			}),
			alu::Inst::Reg(alu::reg::Inst { dst, lhs, rhs, kind }) if dst == lhs => Some(Self {
				dst,
				kind: Kind::Reg { kind, rhs },
			}),
			_ => None,
		}
	}
}

impl InstSize for Inst {
	fn size(&self) -> usize {
		4
	}
}

impl InstFmt for Inst {
	fn mnemonic(&self) -> &'static str {
		self.kind.mnemonic()
	}

	fn fmt(&self, _pos: crate::Pos, _bytes: &[u8], f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let Self { dst, kind } = self;
		let mnemonic = kind.mnemonic();
		let value = kind.value_fmt();

		write!(f, "{mnemonic} {dst}, {value}")
	}
}
