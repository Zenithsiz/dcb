//! Store instructions

// Imports
use super::ModifiesReg;
use crate::inst::{
	basic::{Decode, Encode},
	parse::LineArg,
	DisplayCtx, InstDisplay, InstFmt, InstFmtArg, Parsable, ParseCtx, ParseError, Register,
};
use dcb_util::SignedHex;
use int_conv::{Signed, Truncated, ZeroExtended};
use std::{array, convert::TryInto};

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

impl Decode for Inst {
	#[bitmatch::bitmatch]
	fn decode(raw: u32) -> Option<Self> {
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
impl Encode for Inst {
	#[bitmatch::bitmatch]
	fn encode(&self) -> u32 {
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

impl Parsable for Inst {
	fn parse<Ctx: ?Sized + ParseCtx>(mnemonic: &str, args: &[LineArg], _ctx: &Ctx) -> Result<Self, ParseError> {
		let kind = match mnemonic {
			"sb" => Kind::Byte,
			"sh" => Kind::HalfWord,
			"swl" => Kind::WordLeft,
			"sw" => Kind::Word,
			"swr" => Kind::WordRight,
			_ => return Err(ParseError::UnknownMnemonic),
		};

		let (value, addr, offset) = match *args {
			[LineArg::Register(value), LineArg::Register(addr)] => (value, addr, 0),
			[LineArg::Register(value), LineArg::RegisterOffset { register: addr, offset }] => {
				(value, addr, offset.try_into().map_err(|_| ParseError::LiteralOutOfRange)?)
			},
			_ => return Err(ParseError::InvalidArguments),
		};

		Ok(Self { value, addr, offset, kind })
	}
}

impl InstDisplay for Inst {
	type Args = array::IntoIter<InstFmtArg, 2>;
	type Mnemonic = &'static str;

	fn mnemonic<Ctx: DisplayCtx>(&self, _ctx: &Ctx) -> Self::Mnemonic {
		self.kind.mnemonic()
	}

	fn args<Ctx: DisplayCtx>(&self, _ctx: &Ctx) -> Self::Args {
		let &Self { value, addr, offset, .. } = self;

		array::IntoIter::new([InstFmtArg::Register(value), InstFmtArg::register_offset(addr, offset)])
	}
}

impl InstFmt for Inst {
	fn fmt(&self, _pos: crate::Pos, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let Self { addr, value, offset, kind } = self;
		let mnemonic = kind.mnemonic();

		match offset {
			0 => write!(f, "{mnemonic} {value}, {addr}"),
			_ => write!(f, "{mnemonic} {value}, {:#}({addr})", SignedHex(offset)),
		}
	}
}

impl ModifiesReg for Inst {
	fn modifies_reg(&self, _reg: Register) -> bool {
		false
	}
}
