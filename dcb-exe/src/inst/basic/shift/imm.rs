//! Shift immediate instructions

// Imports
use crate::inst::{
	basic::{Decode, ModifiesReg, TryEncode},
	exec::{ExecCtx, ExecError, Executable},
	parse::LineArg,
	DisplayCtx, InstDisplay, InstFmtArg, Parsable, ParseCtx, ParseError, Register,
};
use int_conv::{Signed, Truncated, ZeroExtended};

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

impl Inst {
	/// NOP
	pub const NOP: Self = Self {
		dst:  Register::Zr,
		lhs:  Register::Zr,
		rhs:  0,
		kind: Kind::LeftLogical,
	};
}

impl Decode for Inst {
	#[bitmatch::bitmatch]
	fn decode(raw: u32) -> Option<Self> {
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

/// Encode error
#[derive(PartialEq, Clone, Debug, thiserror::Error)]
pub enum EncodeError {
	/// Rhs is too large
	#[error("rhs is too large")]
	Rhs,
}

impl TryEncode for Inst {
	type Error = EncodeError;

	#[bitmatch::bitmatch]
	fn try_encode(&self) -> Result<u32, Self::Error> {
		if self.rhs >= 32 {
			return Err(EncodeError::Rhs);
		}

		let f: u32 = match self.kind {
			Kind::LeftLogical => 0x0,
			Kind::RightLogical => 0x2,
			Kind::RightArithmetic => 0x3,
		};
		let t = self.lhs.idx();
		let d = self.dst.idx();
		let i = self.rhs.zero_extended::<u32>();

		Ok(bitpack!("000000_?????_ttttt_ddddd_iiiii_0000ff"))
	}
}

impl<'a> Parsable<'a> for Inst {
	fn parse<Ctx: ?Sized + ParseCtx<'a>>(
		mnemonic: &'a str, args: &'a [LineArg], ctx: &Ctx,
	) -> Result<Self, ParseError> {
		// Special case for `nop`
		if let ("nop", []) = (mnemonic, args) {
			return Ok(Self::NOP);
		}

		let kind = match mnemonic {
			"sll" => Kind::LeftLogical,
			"srl" => Kind::RightLogical,
			"sra" => Kind::RightArithmetic,
			_ => return Err(ParseError::UnknownMnemonic),
		};

		match *args {
			[LineArg::Register(lhs @ dst), LineArg::Expr(ref rhs)] |
			[LineArg::Register(dst), LineArg::Register(lhs), LineArg::Expr(ref rhs)] => Ok(Self {
				dst,
				lhs,
				rhs: ctx.eval_expr_as(rhs)?,
				kind,
			}),
			_ => Err(ParseError::InvalidArguments),
		}
	}
}

impl<'a> InstDisplay<'a> for Inst {
	type Mnemonic = &'static str;

	type Args = impl IntoIterator<Item = InstFmtArg<'a>>;

	fn mnemonic<Ctx: DisplayCtx>(&'a self, _ctx: &Ctx) -> Self::Mnemonic {
		// Special case for `nop`
		if let Self::NOP = *self {
			return "nop";
		}

		self.kind.mnemonic()
	}

	#[auto_enums::auto_enum(Iterator)]
	fn args<Ctx: DisplayCtx>(&'a self, _ctx: &Ctx) -> Self::Args {
		let &Self { dst, lhs, rhs, .. } = self;

		// If `$dst` and `$lhs` are the same, only print one of them
		match dst == lhs {
			// Special case for `nop`
			// Note: This has to be in this match due to `auto_enum`.
			_ if matches!(*self, Self::NOP) => [].into_iter(),

			true => [InstFmtArg::Register(dst), InstFmtArg::literal(rhs)].into_iter(),
			false => [
				InstFmtArg::Register(dst),
				InstFmtArg::Register(lhs),
				InstFmtArg::literal(rhs),
			]
			.into_iter(),
		}
	}
}

impl ModifiesReg for Inst {
	fn modifies_reg(&self, reg: Register) -> bool {
		self.dst == reg
	}
}

impl Executable for Inst {
	fn exec<Ctx: ExecCtx>(&self, state: &mut Ctx) -> Result<(), ExecError> {
		let value = match self.kind {
			Kind::LeftLogical => state.load_reg(self.lhs).wrapping_shl(self.rhs.zero_extended()),
			Kind::RightLogical => state.load_reg(self.lhs).wrapping_shr(self.rhs.zero_extended()),
			Kind::RightArithmetic => state
				.load_reg(self.lhs)
				.as_signed()
				.wrapping_shr(self.rhs.zero_extended())
				.as_unsigned(),
		};
		state.store_reg(self.dst, value);

		Ok(())
	}
}
