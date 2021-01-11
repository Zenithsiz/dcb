//! Shift self-assign instructions

// Imports
use super::Decodable;
use crate::exe::inst::{
	basic::{self, shift},
	InstFmt, InstSize, Register,
};
use std::{convert::TryInto, fmt};

/// Shift assign kind
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Kind {
	/// Immediate
	Imm {
		/// Kind
		kind: shift::imm::Kind,

		/// Argument
		rhs: u8,
	},

	/// Register
	Reg {
		/// Kind
		kind: shift::reg::Kind,

		/// Argument
		rhs: Register,
	},
}

impl Kind {
	/// Returns this kind's mnemonic
	#[must_use]
	pub const fn mnemonic(self) -> &'static str {
		match self {
			Self::Imm { kind, .. } => kind.mnemonic(),
			Self::Reg { kind, .. } => kind.mnemonic(),
		}
	}

	/// Returns a displayable with the value of this kind
	#[must_use]
	pub fn value_fmt(self) -> impl fmt::Display {
		dcb_util::DisplayWrapper::new(move |f| match self {
			Self::Imm { rhs, .. } => write!(f, "{:#x}", rhs),
			Self::Reg { rhs, .. } => write!(f, "{}", rhs),
		})
	}
}

/// Shift self-assign instructions
///
/// Alias for
/// ```mips
/// [shift] $dst, $dst, ...
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
			shift::Inst::Imm(shift::imm::Inst { dst, lhs, rhs, kind }) if dst == lhs => Some(Self {
				dst,
				kind: Kind::Imm { kind, rhs },
			}),
			shift::Inst::Reg(shift::reg::Inst { dst, lhs, rhs, kind }) if dst == lhs => Some(Self {
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
	fn fmt(&self, _pos: crate::Pos, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let Self { dst, kind } = self;
		let mnemonic = kind.mnemonic();
		let value = kind.value_fmt();

		write!(f, "{mnemonic} {dst}, {value}")
	}
}
