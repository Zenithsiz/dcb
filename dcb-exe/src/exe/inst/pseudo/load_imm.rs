//! Load immediate

// Imports
use crate::exe::inst::{
	basic::{alu_imm::AluImmKind, AluImmInst, InstIter},
	BasicInst, Register,
};
use dcb_util::SignedHex;
use std::fmt;

/// Immediate kind
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[derive(derive_more::Display)]
pub enum LoadImmKind {
	/// Address
	///
	/// Alias for `lui $dst, {hi} / addiu $dst, $dst, {lo}`
	Address(u32),

	/// Word
	///
	/// Alias for `lui $dst, {hi} / ori $dst, $dst, {imm-lo}`
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

impl LoadImmKind {
	/// Returns the mnemonic for this load kind
	#[must_use]
	pub const fn mnemonic(self) -> &'static str {
		match self {
			LoadImmKind::Address(_) => "la",
			LoadImmKind::Word(_) | LoadImmKind::HalfWordUnsigned(_) | LoadImmKind::HalfWordSigned(_) => "li",
		}
	}

	/// Returns a displayable with the value of this load kind formatted.
	pub fn value_fmt(self) -> impl fmt::Display {
		#[rustfmt::skip]
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
#[derive(derive_more::Display)]
#[display(fmt = "{} {dst}, {}", "kind.mnemonic()", "kind.value_fmt()")]
pub struct LoadImmInst {
	/// Destination register
	pub dst: Register,

	/// Load kind
	pub kind: LoadImmKind,
}

impl LoadImmInst {
	/// Decodes this pseudo instruction
	#[must_use]
	pub fn decode(iter: InstIter<'_, impl Iterator<Item = u32> + Clone>) -> Option<Self> {
		let peeker = iter.peeker();
		let inst = match peeker.next()?? {
			BasicInst::AluImm(alu @ AluImmInst { lhs: Register::Zr, .. }) => Self {
				dst:  alu.dst,
				kind: match alu.kind {
					AluImmKind::AddUnsigned(rhs) => LoadImmKind::HalfWordSigned(rhs),
					AluImmKind::Or(rhs) => LoadImmKind::HalfWordUnsigned(rhs),
					_ => return None,
				},
			},
			BasicInst::Lui(lui) => match peeker.next()?? {
				BasicInst::AluImm(alu) if lui.dst == alu.dst && lui.dst == alu.lhs => Self {
					dst:  lui.dst,
					kind: match alu.kind {
						AluImmKind::AddUnsigned(rhs) => LoadImmKind::Address(lui.value.zero_extended::<u32>().shl(16) + rhs.sign_extended::<i32>()),
						AluImmKind::Or(rhs) => LoadImmKind::Word(u32::join(rhs, lui.value)),
						_ => return None,
					},
				},
				_ => return None,
			},
			_ => return None,
		};

		peeker.apply();
		return inst;
	}
}
