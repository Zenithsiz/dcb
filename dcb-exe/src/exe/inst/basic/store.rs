//! Store instructions

// Imports
use crate::exe::inst::{
	basic::{Decodable, Encodable},
	InstFmt, Register,
};
use int_conv::{Signed, Truncated, ZeroExtended};

/// Store instruction kind
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Kind {
	/// Byte, `i8`
	Byte,

	/// Half-word, `i16`
	HalfWord,

	/// Word left-bits, `u32`
	WordLeft,

	/// Word, `u32`
	Word,

	/// Word right-bits, `u32`
	WordRight,
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
		let kind = match raw.p {
			0x0 => Kind::Byte,
			0x1 => Kind::HalfWord,
			0x2 => Kind::WordLeft,
			0x3 => Kind::Word,
			0x6 => Kind::WordRight,
			_ => return None,
		};

		Some(Self {
			src: Register::new(raw.t)?,
			dst: Register::new(raw.s)?,
			offset: raw.i.truncated::<u16>().as_signed(),
			kind,
		})
	}
}
impl Encodable for Inst {
	fn encode(&self) -> Raw {
		let p = match self.kind {
			Kind::Byte => 0x0,
			Kind::HalfWord => 0x1,
			Kind::WordLeft => 0x2,
			Kind::Word => 0x3,
			Kind::WordRight => 0x6,
		};
		let t = self.src.idx();
		let s = self.dst.idx();
		let i = self.offset.as_unsigned().zero_extended::<u32>();

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
		let address = pos + 4 * offset;

		write!(f, "{mnemonic} {dst}, {address}({src})")
	}
}
