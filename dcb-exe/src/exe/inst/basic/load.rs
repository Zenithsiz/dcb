//! Load instructions

// Imports
use crate::exe::inst::{
	basic::{Decodable, Encodable},
	InstFmt, Register,
};
use int_conv::{Signed, Truncated, ZeroExtended};
use std::convert::TryFrom;

/// Load instruction kind
///
/// Each variant's value is equal to the lower 3 bits of the opcode
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[derive(num_enum::IntoPrimitive, num_enum::TryFromPrimitive)]
#[repr(u8)]
pub enum Kind {
	/// Byte, `i8`
	Byte             = 0x0,

	/// Half-word, `i16`
	HalfWord         = 0x1,

	/// Word left-bits, `u32`
	WordLeft         = 0x2,

	/// Word, `u32`
	Word             = 0x3,

	/// Byte unsigned, `u8`
	ByteUnsigned     = 0x4,

	/// Half-word unsigned, `u16`
	HalfWordUnsigned = 0x5,

	/// Word right-bits, `u32`
	WordRight        = 0x6,
}

impl Kind {
	/// Returns the mnemonic for this load kind
	#[must_use]
	pub const fn mnemonic(self) -> &'static str {
		match self {
			Self::Byte => "lb",
			Self::HalfWord => "lh",
			Self::WordLeft => "lwl",
			Self::Word => "lw",
			Self::ByteUnsigned => "lbu",
			Self::HalfWordUnsigned => "lhu",
			Self::WordRight => "lwr",
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

/// Load instructions
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
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

impl Decodable for Inst {
	type Raw = Raw;

	fn decode(raw: Self::Raw) -> Option<Self> {
		let op = Kind::try_from(raw.p.truncated::<u8>()).ok()?;

		Some(Self {
			src:    Register::new(raw.t)?,
			dst:    Register::new(raw.s)?,
			offset: raw.i.truncated::<u16>().as_signed(),
			kind:   op,
		})
	}
}

impl Encodable for Inst {
	fn encode(&self) -> Raw {
		let t = self.src.idx();
		let s = self.dst.idx();
		let i = self.offset.as_unsigned().zero_extended::<u32>();
		let p = u8::from(self.kind).zero_extended::<u32>();

		Raw { p, s, t, i }
	}
}

impl InstFmt for Inst {
	fn mnemonic(&self) -> &'static str {
		self.kind.mnemonic()
	}

	fn fmt(&self, pos: crate::Pos, _bytes: &[u8], f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let Self { dst, src, offset, kind } = self;
		let mnemonic = kind.mnemonic();
		let address = pos + *offset;

		write!(f, "{mnemonic} {dst}, {address}({src})")
	}
}
