//! Store instructions

// Imports
use crate::exe::inst::Register;
use dcb_util::SignedHex;
use int_conv::{Signed, Truncated, ZeroExtended};
use std::convert::TryFrom;

/// Store instruction kind
///
/// Each variant's value is equal to the lower 3 bits of the opcode
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[derive(num_enum::IntoPrimitive, num_enum::TryFromPrimitive)]
#[repr(u8)]
pub enum Kind {
	/// Byte, `u8`
	Byte      = 0x0,

	/// Half-word, `u16`
	HalfWord  = 0x1,

	/// Word left-bits, `u32`
	WordLeft  = 0x2,

	/// Word, `u32`
	Word      = 0x3,

	/// Word right-bits, `u32`
	WordRight = 0x6,
}

impl Kind {
	/// Returns the mnemonic for this store kind
	#[must_use]
	pub const fn mnemonic(self) -> &'static str {
		match self {
			Self::Byte => "sb",
			Self::HalfWord => "sh",
			Self::WordLeft => "swl",
			Self::Word => "sw",
			Self::WordRight => "swr",
		}
	}
}

/// Raw representation
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Raw {
	/// Opcode (lower 3 bits)
	pub p: u32,

	/// Rs
	pub s: u32,

	/// Rt
	pub t: u32,

	/// Immediate
	pub i: u32,
}

/// Store instructions
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[derive(derive_more::Display)]
#[display(fmt = "{} {dst}, {}({src})", "kind.mnemonic()", "SignedHex(offset)")]
pub struct Inst {
	/// Source register, `rt`
	pub src: Register,

	/// Destination register, `rs`
	pub dst: Register,

	/// Destination offset.
	pub offset: i16,

	/// Kind
	pub kind: Kind,
}

impl Inst {
	/// Decodes this instruction
	#[must_use]
	pub fn decode(raw: Raw) -> Option<Self> {
		let kind = Kind::try_from(raw.p.truncated::<u8>()).ok()?;

		Some(Self {
			src: Register::new(raw.t)?,
			dst: Register::new(raw.s)?,
			offset: raw.i.truncated::<u16>().as_signed(),
			kind,
		})
	}

	/// Encodes this instruction
	#[must_use]
	pub fn encode(self) -> Raw {
		let t = self.src.idx();
		let s = self.dst.idx();
		let i = self.offset.as_unsigned().zero_extended::<u32>();
		let p = u8::from(self.kind).zero_extended::<u32>();

		Raw { p, s, t, i }
	}
}
