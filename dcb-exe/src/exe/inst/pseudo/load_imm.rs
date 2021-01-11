//! Load immediate

// Imports
use super::Decodable;
use crate::exe::inst::{basic, InstFmt, InstSize, Register};
use dcb_util::SignedHex;
use int_conv::{Join, SignExtended, Signed};
use std::convert::TryInto;

/// Immediate kind
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[derive(derive_more::Display)]
pub enum Kind {
	/// Address
	///
	/// Alias for `lui $dst, {hi} / addiu $dst, $dst, {lo}`
	Address(u32),

	/// Word
	///
	/// Alias for `lui $dst, {hi} / ori $dst, $dst, {lo}`
	Word(u32),

	/// Unsigned half-word
	///
	/// Alias for `ori $dst, $zr, imm`
	HalfWordUnsigned(u16),

	/// Signed half-word
	///
	/// Alias for `addiu $dst, $zr, imm`
	HalfWordSigned(i16),
}

impl Kind {
	/// Returns the mnemonic for this load kind
	#[must_use]
	pub const fn mnemonic(self) -> &'static str {
		match self {
			Self::Address(_) => "la",
			Self::Word(_) | Self::HalfWordUnsigned(_) | Self::HalfWordSigned(_) => "li",
		}
	}

	/// Returns a displayable with the value of this load kind formatted.
	#[rustfmt::skip]
	#[must_use]
	pub fn value_fmt(self) -> impl std::fmt::Display {
		dcb_util::DisplayWrapper::new(move |f| match self {
			Self::Address(address)        => write!(f, "{address:#x}"),
			Self::Word(value)             => write!(f, "{value:#x}"),
			Self::HalfWordUnsigned(value) => write!(f, "{value:#x}"),
			Self::HalfWordSigned(value)   => write!(f, "{}", SignedHex(value)),
		})
	}
}

/// Load immediate instruction
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Inst {
	/// Destination register
	pub dst: Register,

	/// Load kind
	pub kind: Kind,
}

impl Decodable for Inst {
	fn decode(mut insts: impl Iterator<Item = basic::Inst> + Clone) -> Option<Self> {
		use basic::alu::imm::Kind::{AddUnsigned, Or};
		let inst = match insts.next()? {
			// `lui $dst, $value`
			basic::Inst::Lui(lui) => match insts.next()?.try_into().ok()? {
				// Filter for same `$dst` and equal `$dst` and `$lhs`.
				basic::alu::Inst::Imm(alu) if lui.dst == alu.dst && alu.dst == alu.lhs => Self {
					dst:  lui.dst,
					kind: match alu.kind {
						// lui << 16 + rhs
						AddUnsigned(rhs) => Kind::Address((u32::join(0, lui.value).as_signed() + rhs.sign_extended::<i32>()).as_unsigned()),
						Or(rhs) => Kind::Word(u32::join(rhs, lui.value)),
						_ => return None,
					},
				},
				_ => return None,
			},
			// `addiu $zr, $value`
			// `ori   $zr, $value`
			#[rustfmt::skip]
			basic::Inst::Alu(basic::alu::Inst::Imm(inst)) if inst.lhs == Register::Zr => Self {
				dst:        inst.dst,
				kind: match inst.kind {
					AddUnsigned(value) => Kind::HalfWordSigned  (value),
					Or         (value) => Kind::HalfWordUnsigned(value),
					_ => return None,
				},
			},

			_ => return None,
		};

		Some(inst)
	}
}

impl InstSize for Inst {
	fn size(&self) -> usize {
		match self.kind {
			Kind::Address(_) | Kind::Word(_) => 8,
			Kind::HalfWordUnsigned(_) | Kind::HalfWordSigned(_) => 4,
		}
	}
}

impl InstFmt for Inst {
	fn mnemonic(&self) -> &'static str {
		self.kind.mnemonic()
	}

	fn fmt(&self, _pos: crate::Pos, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let Self { dst, kind } = self;
		let mnemonic = kind.mnemonic();
		let value = kind.value_fmt();

		write!(f, "{mnemonic} {dst}, {value}")
	}
}
