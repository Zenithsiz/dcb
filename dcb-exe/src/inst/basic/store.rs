//! Store instructions

// Imports
use super::ModifiesReg;
use crate::inst::{
	basic::{Decodable, Encodable},
	InstFmt, Register,
};
use dcb_util::SignedHex;
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

/// Store instructions
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Inst {
	/// Value register, `rt`
	pub value: Register,

	/// Address register, `rs`
	pub addr: Register,

	/// Address offset.
	pub offset: i16,

	/// Kind
	pub kind: Kind,
}

impl Decodable for Inst {
	type Raw = u32;

	#[bitmatch::bitmatch]
	fn decode(raw: Self::Raw) -> Option<Self> {
		let [p, s, t, i] = #[bitmatch]
		match raw {
			"101ppp_sssss_ttttt_iiiii_iiiii_iiiiii" => [p, s, t, i],
			_ => return None,
		};

		let kind = match p {
			0x0 => Kind::Byte,
			0x1 => Kind::HalfWord,
			0x2 => Kind::WordLeft,
			0x3 => Kind::Word,
			0x6 => Kind::WordRight,
			_ => return None,
		};

		Some(Self {
			value: Register::new(t)?,
			addr: Register::new(s)?,
			offset: i.truncated::<u16>().as_signed(),
			kind,
		})
	}
}
impl Encodable for Inst {
	#[bitmatch::bitmatch]
	fn encode(&self) -> Self::Raw {
		let p: u32 = match self.kind {
			Kind::Byte => 0x0,
			Kind::HalfWord => 0x1,
			Kind::WordLeft => 0x2,
			Kind::Word => 0x3,
			Kind::WordRight => 0x6,
		};
		let t = self.value.idx();
		let s = self.addr.idx();
		let i = self.offset.as_unsigned().zero_extended::<u32>();

		bitpack!("101ppp_sssss_ttttt_iiiii_iiiii_iiiiii")
	}
}

impl InstFmt for Inst {
	fn fmt(&self, _pos: crate::Pos, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let Self { addr, value, offset, kind } = self;
		let mnemonic = kind.mnemonic();

		write!(f, "{mnemonic} {value}, {:#}({addr})", SignedHex(offset))
	}
}

impl ModifiesReg for Inst {
	fn modifies_reg(&self, _reg: Register) -> bool {
		false
	}
}
