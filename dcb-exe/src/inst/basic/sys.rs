//! System calls

// Imports
use super::ModifiesReg;
use crate::inst::{
	basic::{Decode, TryEncode},
	exec::{ExecCtx, ExecError, Executable},
	parse::LineArg,
	DisplayCtx, InstDisplay, InstFmtArg, Parsable, ParseCtx, ParseError, Register,
};
use std::array;

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

impl Decode for Inst {
	#[bitmatch::bitmatch]
	fn decode(raw: u32) -> Option<Self> {
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

/// Encode error
#[derive(PartialEq, Clone, Debug, thiserror::Error)]
pub enum EncodeError {
	/// Comment is too large
	#[error("Comment is too large")]
	Comment,
}

impl TryEncode for Inst {
	type Error = EncodeError;

	#[bitmatch::bitmatch]
	fn try_encode(&self) -> Result<u32, Self::Error> {
		if self.comment >= 0x100000 {
			return Err(EncodeError::Comment);
		}

		let c = self.comment;
		let f: u32 = match self.kind {
			Kind::Sys => 0,
			Kind::Break => 1,
		};

		Ok(bitpack!("000000_ccccc_ccccc_ccccc_ccccc_00110f"))
	}
}

impl<'a> Parsable<'a> for Inst {
	fn parse<Ctx: ?Sized + ParseCtx<'a>>(
		mnemonic: &'a str, args: &'a [LineArg], ctx: &Ctx,
	) -> Result<Self, ParseError> {
		let kind = match mnemonic {
			"sys" => Kind::Sys,
			"break" => Kind::Break,
			_ => return Err(ParseError::UnknownMnemonic),
		};

		let comment = match args {
			[LineArg::Expr(comment)] => ctx.eval_expr_as(comment)?,
			_ => return Err(ParseError::InvalidArguments),
		};

		Ok(Self { comment, kind })
	}
}

impl<'a> InstDisplay<'a> for Inst {
	type Args = array::IntoIter<InstFmtArg<'a>, 1>;
	type Mnemonic = &'static str;

	fn mnemonic<Ctx: DisplayCtx>(&'a self, _ctx: &Ctx) -> Self::Mnemonic {
		self.kind.mnemonic()
	}

	fn args<Ctx: DisplayCtx>(&'a self, _ctx: &Ctx) -> Self::Args {
		array::IntoIter::new([InstFmtArg::literal(self.comment)])
	}
}

impl ModifiesReg for Inst {
	fn modifies_reg(&self, _reg: Register) -> bool {
		false
	}
}

impl Executable for Inst {
	fn exec<Ctx: ExecCtx>(&self, state: &mut Ctx) -> Result<(), ExecError> {
		state.sys(*self)
	}
}
