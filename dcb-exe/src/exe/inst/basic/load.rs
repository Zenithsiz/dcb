//! Load instructions

// Imports
use crate::exe::inst::{
	basic::{Decodable, Encodable},
	InstFmt, Register,
};
use dcb_util::SignedHex;
use int_conv::{Signed, Truncated, ZeroExtended};

/// Load instruction kind
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

	/// Byte unsigned, `u8`
	ByteUnsigned,

	/// Half-word unsigned, `u16`
	HalfWordUnsigned,

	/// Word right-bits, `u32`
	WordRight,
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
		let kind = match raw.p {
			0x0 => Kind::Byte,
			0x1 => Kind::HalfWord,
			0x2 => Kind::WordLeft,
			0x3 => Kind::Word,
			0x4 => Kind::ByteUnsigned,
			0x5 => Kind::HalfWordUnsigned,
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
	fn encode(&self) -> Self::Raw {
		let p = match self.kind {
			Kind::Byte => 0x0,
			Kind::HalfWord => 0x1,
			Kind::WordLeft => 0x2,
			Kind::Word => 0x3,
			Kind::ByteUnsigned => 0x4,
			Kind::HalfWordUnsigned => 0x5,
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

	fn fmt(&self, _pos: crate::Pos, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let Self { dst, src, offset, kind } = self;
		let mnemonic = kind.mnemonic();

		write!(f, "{mnemonic} {dst}, {:#}({src})", SignedHex(offset))
	}
}
