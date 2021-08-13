//! Move register instruction

// Imports
use super::{Decodable, Encodable};
use crate::inst::{
	basic, parse::LineArg, DisplayCtx, InstDisplay, InstFmtArg, InstSize, Parsable, ParseCtx, ParseError, Register,
};
use std::convert::TryInto;

/// Move register instruction
///
/// Alias for
/// ```mips
/// addu $dst, $src, $zr
/// ```
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Inst {
	/// Destination register
	pub dst: Register,

	/// Source register
	pub src: Register,
}


impl Decodable for Inst {
	fn decode(mut insts: impl Iterator<Item = basic::Inst> + Clone) -> Option<Self> {
		match insts.next()?.try_into().ok()? {
			basic::alu::Inst::Reg(basic::alu::reg::Inst {
				dst,
				lhs,
				rhs: Register::Zr,
				kind: basic::alu::reg::Kind::AddUnsigned,
			}) => Some(Self { dst, src: lhs }),
			_ => None,
		}
	}
}

impl<'a> Encodable<'a> for Inst {
	type Iterator = impl Iterator<Item = basic::Inst> + 'a;

	fn encode(&'a self) -> Self::Iterator {
		std::iter::once(basic::Inst::Alu(basic::alu::Inst::Reg(basic::alu::reg::Inst {
			dst:  self.dst,
			lhs:  self.src,
			rhs:  Register::Zr,
			kind: basic::alu::reg::Kind::AddUnsigned,
		})))
	}
}

impl<'a> Parsable<'a> for Inst {
	fn parse<Ctx: ?Sized + ParseCtx<'a>>(
		mnemonic: &'a str, args: &'a [LineArg], _ctx: &Ctx,
	) -> Result<Self, ParseError> {
		if mnemonic != "move" {
			return Err(ParseError::UnknownMnemonic);
		}

		let (dst, src) = match *args {
			[LineArg::Register(dst), LineArg::Register(src)] => (dst, src),
			_ => return Err(ParseError::InvalidArguments),
		};

		Ok(Self { dst, src })
	}
}

impl<'a> InstDisplay<'a> for Inst {
	type Args = [InstFmtArg<'a>; 2];
	type Mnemonic = &'static str;

	fn mnemonic<Ctx: DisplayCtx>(&'a self, _ctx: &Ctx) -> Self::Mnemonic {
		"move"
	}

	fn args<Ctx: DisplayCtx>(&'a self, _ctx: &Ctx) -> Self::Args {
		let &Self { dst, src } = self;

		[InstFmtArg::Register(dst), InstFmtArg::Register(src)]
	}
}

impl InstSize for Inst {
	fn size(&self) -> usize {
		4
	}
}
