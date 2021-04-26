//! System calls

// Imports
use super::{ModifiesReg, Parsable, ParseError};
use crate::inst::{
	basic::{Decodable, Encodable},
	parse::LineArg,
	InstFmt, ParseCtx, Register,
};
use std::convert::TryInto;

/// Sys instruction func
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Kind {
	/// Syscall
	Sys,

	/// Break
	Break,
}

impl Kind {
	/// Returns the mnemonic associated with this syscall kind
	#[must_use]
	pub const fn mnemonic(self) -> &'static str {
		match self {
			Self::Sys => "sys",
			Self::Break => "break",
		}
	}
}

/// Syscall instructions
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Inst {
	/// Comment
	pub comment: u32,

	/// Kind
	pub kind: Kind,
}

impl Decodable for Inst {
	type Raw = u32;

	#[bitmatch::bitmatch]
	fn decode(raw: Self::Raw) -> Option<Self> {
		let [c, f] = #[bitmatch]
		match raw {
			"000000_ccccc_ccccc_ccccc_ccccc_00110f" => [c, f],
			_ => return None,
		};

		let kind = match f {
			0 => Kind::Sys,
			1 => Kind::Break,
			_ => return None,
		};

		Some(Self { comment: c, kind })
	}
}

impl Encodable for Inst {
	#[bitmatch::bitmatch]
	fn encode(&self) -> Self::Raw {
		// TODO: Maybe return Error?
		assert!(self.comment < 0x100000);

		let c = self.comment;
		let f: u32 = match self.kind {
			Kind::Sys => 0,
			Kind::Break => 1,
		};

		bitpack!("000000_ccccc_ccccc_ccccc_ccccc_00110f")
	}
}


impl Parsable for Inst {
	fn parse<Ctx: ?Sized + ParseCtx>(mnemonic: &str, args: &[LineArg], _ctx: &Ctx) -> Result<Self, ParseError> {
		let kind = match mnemonic {
			"sys" => Kind::Sys,
			"break" => Kind::Break,
			_ => return Err(ParseError::UnknownMnemonic),
		};

		let comment = match *args {
			[LineArg::Literal(comment)] => comment.try_into().map_err(|_| ParseError::LiteralOutOfRange)?,
			_ => return Err(ParseError::InvalidArguments),
		};

		Ok(Self { comment, kind })
	}
}


impl InstFmt for Inst {
	fn fmt(&self, _pos: crate::Pos, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let Self { comment, kind } = self;
		let mnemonic = kind.mnemonic();

		write!(f, "{mnemonic} {comment:#x}")
	}
}

impl ModifiesReg for Inst {
	fn modifies_reg(&self, _reg: Register) -> bool {
		false
	}
}
