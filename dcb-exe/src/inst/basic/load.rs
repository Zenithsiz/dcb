//! Load instructions

// Imports
use super::ModifiesReg;
use crate::{
	inst::{
		basic::{Decode, Encode},
		exec::{ExecCtx, ExecError, Executable},
		parse::LineArg,
		DisplayCtx, InstDisplay, InstFmtArg, Parsable, ParseCtx, ParseError, Register,
	},
	Pos,
};
use int_conv::{SignExtended, Signed, Truncated, ZeroExtended};
use std::array;

/// Instruction kind
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
	/// Returns the mnemonic for this kind
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

	/// Returns this kind from it's mnemonic
	#[must_use]
	pub fn from_mnemonic(mnemonic: &str) -> Option<Self> {
		let kind = match mnemonic {
			"lb" => Self::Byte,
			"lh" => Self::HalfWord,
			"lwl" => Self::WordLeft,
			"lw" => Self::Word,
			"lbu" => Self::ByteUnsigned,
			"lhu" => Self::HalfWordUnsigned,
			"lwr" => Self::WordRight,
			_ => return None,
		};
		Some(kind)
	}
}

/// Load instructions
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
			"100ppp_sssss_ttttt_iiiii_iiiii_iiiiii" => [p, s, t, i],
			_ => return None,
		};

		Some(Self {
			value:  Register::new(t)?,
			addr:   Register::new(s)?,
			offset: i.truncated::<u16>().as_signed(),
			kind:   match p {
				0x0 => Kind::Byte,
				0x1 => Kind::HalfWord,
				0x2 => Kind::WordLeft,
				0x3 => Kind::Word,
				0x4 => Kind::ByteUnsigned,
				0x5 => Kind::HalfWordUnsigned,
				0x6 => Kind::WordRight,
				_ => return None,
			},
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
			Kind::ByteUnsigned => 0x4,
			Kind::HalfWordUnsigned => 0x5,
			Kind::WordRight => 0x6,
		};
		let t = self.value.idx();
		let s = self.addr.idx();
		let i = self.offset.as_unsigned().zero_extended::<u32>();

		bitpack!("100ppp_sssss_ttttt_iiiii_iiiii_iiiiii")
	}
}

impl<'a> Parsable<'a> for Inst {
	fn parse<Ctx: ?Sized + ParseCtx<'a>>(
		mnemonic: &'a str, args: &'a [LineArg], ctx: &Ctx,
	) -> Result<Self, ParseError> {
		let kind = Kind::from_mnemonic(mnemonic).ok_or(ParseError::UnknownMnemonic)?;

		let (value, addr, offset) = match *args {
			[LineArg::Register(value), LineArg::Register(addr)] => (value, addr, 0),
			[LineArg::Register(value), LineArg::RegisterOffset {
				register: addr,
				ref offset,
			}] => (value, addr, ctx.eval_expr_as(offset)?),
			_ => return Err(ParseError::InvalidArguments),
		};

		Ok(Self {
			value,
			addr,
			offset,
			kind,
		})
	}
}

impl<'a> InstDisplay<'a> for Inst {
	type Args = array::IntoIter<InstFmtArg<'a>, 2>;
	type Mnemonic = &'static str;

	fn mnemonic<Ctx: DisplayCtx>(&'a self, _ctx: &Ctx) -> Self::Mnemonic {
		self.kind.mnemonic()
	}

	fn args<Ctx: DisplayCtx>(&'a self, _ctx: &Ctx) -> Self::Args {
		let &Self {
			value, addr, offset, ..
		} = self;

		array::IntoIter::new([InstFmtArg::Register(value), InstFmtArg::register_offset(addr, offset)])
	}
}

impl ModifiesReg for Inst {
	fn modifies_reg(&self, reg: Register) -> bool {
		self.value == reg
	}
}


impl Executable for Inst {
	fn exec<Ctx: ExecCtx>(&self, state: &mut Ctx) -> Result<(), ExecError> {
		let value = match self.kind {
			Kind::Byte => state
				.read_byte(Pos(state.load_reg(self.addr)))?
				.as_signed()
				.sign_extended::<i32>()
				.as_unsigned(),
			Kind::HalfWord => state
				.read_half_word(Pos(state.load_reg(self.addr)))?
				.as_signed()
				.sign_extended::<i32>()
				.as_unsigned(),
			Kind::Word => state.read_word(Pos(state.load_reg(self.addr)))?,
			Kind::ByteUnsigned => state.read_byte(Pos(state.load_reg(self.addr)))?.zero_extended(),
			Kind::HalfWordUnsigned => state.read_half_word(Pos(state.load_reg(self.addr)))?.zero_extended(),
			Kind::WordLeft | Kind::WordRight => todo!(),
		};
		state.store_reg(self.value, value);

		Ok(())
	}
}
