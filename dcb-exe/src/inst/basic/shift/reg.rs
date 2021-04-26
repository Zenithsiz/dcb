//! Shift register instructions

// Imports
use crate::inst::{
	basic::{Decodable, Encodable, ModifiesReg, Parsable, ParseError},
	parse, InstFmt, ParseCtx, Register,
};

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

impl Decodable for Inst {
	type Raw = u32;

	#[bitmatch::bitmatch]
	fn decode(raw: Self::Raw) -> Option<Self> {
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

impl Encodable for Inst {
	#[bitmatch::bitmatch]
	fn encode(&self) -> Self::Raw {
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

impl Parsable for Inst {
	fn parse<Ctx: ?Sized + ParseCtx>(mnemonic: &str, args: &[parse::Arg], _ctx: &Ctx) -> Result<Self, ParseError> {
		let kind = match mnemonic {
			"sllv" => Kind::LeftLogical,
			"srlv" => Kind::RightLogical,
			"srav" => Kind::RightArithmetic,
			_ => return Err(ParseError::UnknownMnemonic),
		};

		match *args {
			[parse::Arg::Register(lhs @ dst), parse::Arg::Register(rhs)] |
			[parse::Arg::Register(dst), parse::Arg::Register(lhs), parse::Arg::Register(rhs)] => Ok(Self { dst, lhs, rhs, kind }),
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
			true => write!(f, "{mnemonic} {dst}, {rhs}"),
			false => write!(f, "{mnemonic} {dst}, {lhs}, {rhs}"),
		}
	}
}

impl ModifiesReg for Inst {
	fn modifies_reg(&self, reg: Register) -> bool {
		self.dst == reg
	}
}
