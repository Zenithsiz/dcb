//! Shift register instructions

// Imports
use crate::inst::{
	basic::{Decode, Encode, ModifiesReg},
	parse::LineArg,
	DisplayCtx, InstDisplay, InstFmtArg, Parsable, ParseCtx, ParseError, Register,
};
use std::array;

/// Shift register instruction kind
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Kind {
	/// Left logical
	LeftLogical,

	/// Right logical
	RightLogical,

	/// Right arithmetic
	RightArithmetic,
}

impl Kind {
	/// Returns this kind's mnemonic
	#[must_use]
	pub const fn mnemonic(self) -> &'static str {
		match self {
			Self::LeftLogical => "sllv",
			Self::RightLogical => "srlv",
			Self::RightArithmetic => "srav",
		}
	}
}

/// Shift register instructions
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Inst {
	/// Destination register
	pub dst: Register,

	/// Lhs argument
	pub lhs: Register,

	/// Rhs argument
	pub rhs: Register,

	/// Kind
	pub kind: Kind,
}

impl Decode for Inst {
	#[bitmatch::bitmatch]
	fn decode(raw: u32) -> Option<Self> {
		let [s, t, d, f] = #[bitmatch]
		match raw {
			"000000_sssss_ttttt_ddddd_?????_0001ff" => [s, t, d, f],
			_ => return None,
		};

		let kind = match f {
			0x0 => Kind::LeftLogical,
			0x2 => Kind::RightLogical,
			0x3 => Kind::RightArithmetic,
			_ => return None,
		};

		Some(Self {
			dst: Register::new(d)?,
			lhs: Register::new(t)?,
			rhs: Register::new(s)?,
			kind,
		})
	}
}

impl Encode for Inst {
	#[bitmatch::bitmatch]
	fn encode(&self) -> u32 {
		let f: u32 = match self.kind {
			Kind::LeftLogical => 0x0,
			Kind::RightLogical => 0x2,
			Kind::RightArithmetic => 0x3,
		};

		let d = self.dst.idx();
		let t = self.lhs.idx();
		let s = self.rhs.idx();

		bitpack!("000000_sssss_ttttt_ddddd_?????_0001ff")
	}
}

impl<'a> Parsable<'a> for Inst {
	fn parse<Ctx: ?Sized + ParseCtx>(mnemonic: &'a str, args: &'a [LineArg], _ctx: &'a Ctx) -> Result<Self, ParseError> {
		let kind = match mnemonic {
			"sllv" => Kind::LeftLogical,
			"srlv" => Kind::RightLogical,
			"srav" => Kind::RightArithmetic,
			_ => return Err(ParseError::UnknownMnemonic),
		};

		match *args {
			[LineArg::Register(lhs @ dst), LineArg::Register(rhs)] | [LineArg::Register(dst), LineArg::Register(lhs), LineArg::Register(rhs)] => {
				Ok(Self { dst, lhs, rhs, kind })
			},
			_ => Err(ParseError::InvalidArguments),
		}
	}
}

impl<'a> InstDisplay<'a> for Inst {
	type Mnemonic = &'static str;

	type Args = impl IntoIterator<Item = InstFmtArg<'a>>;

	fn mnemonic<Ctx: DisplayCtx>(&'a self, _ctx: &Ctx) -> Self::Mnemonic {
		self.kind.mnemonic()
	}

	#[auto_enums::auto_enum(Iterator)]
	fn args<Ctx: DisplayCtx>(&'a self, _ctx: &Ctx) -> Self::Args {
		let &Self { dst, lhs, rhs, .. } = self;

		// If `$dst` and `$lhs` are the same, only print one of them
		match dst == lhs {
			true => array::IntoIter::new([InstFmtArg::Register(dst), InstFmtArg::Register(rhs)]),
			false => array::IntoIter::new([InstFmtArg::Register(dst), InstFmtArg::Register(lhs), InstFmtArg::Register(rhs)]),
		}
	}
}

impl ModifiesReg for Inst {
	fn modifies_reg(&self, reg: Register) -> bool {
		self.dst == reg
	}
}
