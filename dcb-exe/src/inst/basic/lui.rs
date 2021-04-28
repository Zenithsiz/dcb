//! Lui instruction

// Imports
use super::ModifiesReg;
use crate::inst::{
	basic::{Decode, Encode},
	parse::LineArg,
	DisplayCtx, InstDisplay, InstFmtArg, Parsable, ParseCtx, ParseError, Register,
};
use int_conv::{Truncated, ZeroExtended};
use std::array;

/// Load instructions
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Inst {
	/// Destination register, `rt`
	pub dst: Register,

	/// Value
	pub value: u16,
}

impl Decode for Inst {
	#[bitmatch::bitmatch]
	fn decode(raw: u32) -> Option<Self> {
		let [t, i] = #[bitmatch]
		match raw {
			"001111_?????_ttttt_iiiii_iiiii_iiiiii" => [t, i],
			_ => return None,
		};

		Some(Self {
			dst:   Register::new(t)?,
			value: i.truncated::<u16>(),
		})
	}
}

impl Encode for Inst {
	#[bitmatch::bitmatch]
	fn encode(&self) -> u32 {
		let t = self.dst.idx();
		let i = self.value.zero_extended::<u32>();

		bitpack!("001111_?????_ttttt_iiiii_iiiii_iiiiii")
	}
}

impl<'a> Parsable<'a> for Inst {
	fn parse<Ctx: ?Sized + ParseCtx>(mnemonic: &'a str, args: &'a [LineArg], ctx: &'a Ctx) -> Result<Self, ParseError> {
		if mnemonic != "lui" {
			return Err(ParseError::UnknownMnemonic);
		}

		match *args {
			[LineArg::Register(dst), LineArg::Expr(ref expr)] => Ok(Self {
				dst,
				value: ctx.eval_expr_as(expr)?,
			}),
			_ => Err(ParseError::InvalidArguments),
		}
	}
}

impl<'a> InstDisplay<'a> for Inst {
	type Args = array::IntoIter<InstFmtArg<'a>, 2>;
	type Mnemonic = &'static str;

	fn mnemonic<Ctx: DisplayCtx>(&'a self, _ctx: &Ctx) -> Self::Mnemonic {
		"lui"
	}

	fn args<Ctx: DisplayCtx>(&'a self, _ctx: &Ctx) -> Self::Args {
		let &Self { dst, value } = self;

		array::IntoIter::new([InstFmtArg::Register(dst), InstFmtArg::literal(value)])
	}
}

impl ModifiesReg for Inst {
	fn modifies_reg(&self, reg: Register) -> bool {
		self.dst == reg
	}
}
