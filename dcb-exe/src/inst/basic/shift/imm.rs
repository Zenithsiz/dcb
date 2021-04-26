//! Shift immediate instructions

// Imports
use crate::inst::{
	basic::{Decodable, Encodable, ModifiesReg, Parsable, ParseError},
	parse, InstFmt, ParseCtx, Register,
};
use int_conv::{Truncated, ZeroExtended};
use std::convert::TryInto;

/// Shift immediate instruction kind
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
			Self::LeftLogical => "sll",
			Self::RightLogical => "srl",
			Self::RightArithmetic => "sra",
		}
	}
}

/// Shift immediate instructions
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Inst {
	/// Destination register
	pub dst: Register,

	/// Lhs argument
	pub lhs: Register,

	/// Rhs argument
	pub rhs: u8,

	/// Kind
	pub kind: Kind,
}

impl Decodable for Inst {
	type Raw = u32;

	#[bitmatch::bitmatch]
	fn decode(raw: Self::Raw) -> Option<Self> {
		let [t, d, i, f] = #[bitmatch]
		match raw {
			"000000_?????_ttttt_ddddd_iiiii_0000ff" => [t, d, i, f],
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
			rhs: i.truncated(),
			kind,
		})
	}
}

impl Encodable for Inst {
	#[bitmatch::bitmatch]
	fn encode(&self) -> Self::Raw {
		// TODO: Maybe return error?
		assert!(self.rhs < 32);

		let f: u32 = match self.kind {
			Kind::LeftLogical => 0x0,
			Kind::RightLogical => 0x2,
			Kind::RightArithmetic => 0x3,
		};
		let t = self.lhs.idx();
		let d = self.dst.idx();
		let i = self.rhs.zero_extended::<u32>();

		bitpack!("000000_?????_ttttt_ddddd_iiiii_0000ff")
	}
}

impl Parsable for Inst {
	fn parse<Ctx: ?Sized + ParseCtx>(mnemonic: &str, args: &[parse::Arg], _ctx: &Ctx) -> Result<Self, ParseError> {
		let kind = match mnemonic {
			"sll" => Kind::LeftLogical,
			"srl" => Kind::RightLogical,
			"sra" => Kind::RightArithmetic,
			_ => return Err(ParseError::UnknownMnemonic),
		};

		match *args {
			[parse::Arg::Register(lhs @ dst), parse::Arg::Literal(rhs)] |
			[parse::Arg::Register(dst), parse::Arg::Register(lhs), parse::Arg::Literal(rhs)] => Ok(Self {
				dst,
				lhs,
				rhs: rhs.try_into().map_err(|_| ParseError::LiteralOutOfRange)?,
				kind,
			}),
			_ => Err(ParseError::InvalidArguments),
		}
	}
}

impl InstFmt for Inst {
	fn fmt(&self, _pos: crate::Pos, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let Self { dst, lhs, rhs, kind } = self;
		let mnemonic = kind.mnemonic();

		// If `$dst` and `$lhs` are the same, only print one of them
		match dst == lhs {
			true => write!(f, "{mnemonic} {dst}, {rhs:#x}"),
			false => write!(f, "{mnemonic} {dst}, {lhs}, {rhs:#x}"),
		}
	}
}

impl ModifiesReg for Inst {
	fn modifies_reg(&self, reg: Register) -> bool {
		self.dst == reg
	}
}
