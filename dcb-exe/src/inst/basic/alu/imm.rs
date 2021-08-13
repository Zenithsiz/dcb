//! Alu immediate instructions

// Imports
use crate::inst::{
	basic::{Decode, Encode, ModifiesReg},
	exec::{ExecCtx, ExecError, Executable},
	parse::LineArg,
	DisplayCtx, InstDisplay, InstFmtArg, Parsable, ParseCtx, ParseError, Register,
};
use int_conv::{SignExtended, Signed, Truncated, ZeroExtended};
use std::{array, convert::TryInto};

/// Instruction kind
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Kind {
	/// Add signed with overflow trap
	Add(i16),

	/// Add signed without overflow trap
	AddUnsigned(i16),

	/// Set on less than signed
	SetLessThan(i16),

	/// Set on less than unsigned
	SetLessThanUnsigned(u16),

	/// Bit and
	And(u16),

	/// Bit or
	Or(u16),

	/// Bit xor
	Xor(u16),
}

impl Kind {
	/// Returns this kind's mnemonic
	#[must_use]
	pub const fn mnemonic(self) -> &'static str {
		match self {
			Self::Add(_) => "addi",
			Self::AddUnsigned(_) => "addiu",
			Self::SetLessThan(_) => "slti",
			Self::SetLessThanUnsigned(_) => "sltiu",
			Self::And(_) => "andi",
			Self::Or(_) => "ori",
			Self::Xor(_) => "xori",
		}
	}

	/// Returns the value of this kind as a `i64`
	#[must_use]
	pub fn value(self) -> i64 {
		match self {
			Kind::Add(value) | Kind::AddUnsigned(value) | Kind::SetLessThan(value) => i64::from(value),
			Kind::SetLessThanUnsigned(value) | Kind::And(value) | Kind::Or(value) | Kind::Xor(value) => {
				i64::from(value)
			},
		}
	}
}

/// Alu immediate instructions
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Inst {
	/// Destination register
	pub dst: Register,

	/// Lhs argument
	pub lhs: Register,

	/// Kind
	pub kind: Kind,
}

impl Decode for Inst {
	#[bitmatch::bitmatch]
	fn decode(raw: u32) -> Option<Self> {
		let [p, s, t, i] = #[bitmatch]
		match raw {
			"001ppp_sssss_ttttt_iiiii_iiiii_iiiiii" => [p, s, t, i],
			_ => return None,
		};

		Some(Self {
			dst:  Register::new(t)?,
			lhs:  Register::new(s)?,
			kind: match p {
				0x0 => Kind::Add(i.truncated::<u16>().as_signed()),
				0x1 => Kind::AddUnsigned(i.truncated::<u16>().as_signed()),
				0x2 => Kind::SetLessThan(i.truncated::<u16>().as_signed()),
				0x3 => Kind::SetLessThanUnsigned(i.truncated::<u16>()),
				0x4 => Kind::And(i.truncated::<u16>()),
				0x5 => Kind::Or(i.truncated::<u16>()),
				0x6 => Kind::Xor(i.truncated::<u16>()),
				_ => return None,
			},
		})
	}
}

impl Encode for Inst {
	#[bitmatch::bitmatch]
	fn encode(&self) -> u32 {
		#[rustfmt::skip]
		let (p, i): (u32, u32) = match self.kind {
			Kind::Add                (rhs) => (0x0, rhs.as_unsigned().zero_extended::<u32>()),
			Kind::AddUnsigned        (rhs) => (0x1, rhs.as_unsigned().zero_extended::<u32>()),
			Kind::SetLessThan        (rhs) => (0x2, rhs.as_unsigned().zero_extended::<u32>()),
			Kind::SetLessThanUnsigned(rhs) => (0x3, rhs              .zero_extended::<u32>()),
			Kind::And                (rhs) => (0x4, rhs              .zero_extended::<u32>()),
			Kind::Or                 (rhs) => (0x5, rhs              .zero_extended::<u32>()),
			Kind::Xor                (rhs) => (0x6, rhs              .zero_extended::<u32>()),
		};
		let t = self.dst.idx();
		let s = self.lhs.idx();

		bitpack!("001ppp_sssss_ttttt_iiiii_iiiii_iiiiii")
	}
}

impl<'a> Parsable<'a> for Inst {
	fn parse<Ctx: ?Sized + ParseCtx<'a>>(
		mnemonic: &'a str, args: &'a [LineArg], ctx: &Ctx,
	) -> Result<Self, ParseError> {
		#[rustfmt::skip]
		let to_kind = match mnemonic {
			"addi"  => |value: i64| value.try_into().map(Kind::Add                ),
			"addiu" => |value: i64| value.try_into().map(Kind::AddUnsigned        ),
			"slti"  => |value: i64| value.try_into().map(Kind::SetLessThan        ),
			"sltiu" => |value: i64| value.try_into().map(Kind::SetLessThanUnsigned),
			"andi"  => |value: i64| value.try_into().map(Kind::And                ),
			"ori"   => |value: i64| value.try_into().map(Kind::Or                 ),
			"xori"  => |value: i64| value.try_into().map(Kind::Xor                ),
			_ => return Err(ParseError::UnknownMnemonic),
		};

		match *args {
			// Disallow `slti` and `sltiu` in short form
			[LineArg::Register(_), LineArg::Expr(_)] if ["slti", "sltiu"].contains(&mnemonic) => {
				Err(ParseError::InvalidArguments)
			},

			// Else parse both `$dst, $lhs, value` and `$dst, value`.
			[LineArg::Register(lhs @ dst), LineArg::Expr(ref expr)] |
			[LineArg::Register(dst), LineArg::Register(lhs), LineArg::Expr(ref expr)] => Ok(Self {
				dst,
				lhs,
				kind: to_kind(ctx.eval_expr(expr)?).map_err(|_| ParseError::LiteralOutOfRange)?,
			}),
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
		let &Self { dst, lhs, kind } = self;
		let value = kind.value();

		// If we're not `slti[u]` and if `$dst` and `$lhs` are the same,
		// only return one of them
		match !matches!(kind, Kind::SetLessThan(_) | Kind::SetLessThanUnsigned(_)) && dst == lhs {
			true => array::IntoIter::new([InstFmtArg::Register(dst), InstFmtArg::literal(value)]),
			false => array::IntoIter::new([
				InstFmtArg::Register(dst),
				InstFmtArg::Register(lhs),
				InstFmtArg::literal(value),
			]),
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
			Kind::Add(rhs) => state
				.load_reg(self.lhs)
				.as_signed()
				.checked_add(rhs.sign_extended::<i32>())
				.ok_or(ExecError::Overflow)?
				.as_unsigned(),
			Kind::AddUnsigned(rhs) => state
				.load_reg(self.lhs)
				.as_signed()
				.wrapping_add(rhs.sign_extended::<i32>())
				.as_unsigned(),
			Kind::SetLessThan(rhs) => state
				.load_reg(self.lhs)
				.as_signed()
				.lt(&rhs.sign_extended::<i32>())
				.into(),
			// TODO: Verify it's sign extended
			Kind::SetLessThanUnsigned(rhs) => state
				.load_reg(self.lhs)
				.lt(&rhs.as_signed().sign_extended::<i32>().as_unsigned())
				.into(),
			Kind::And(rhs) => state.load_reg(self.lhs) & rhs.zero_extended::<u32>(),
			Kind::Or(rhs) => state.load_reg(self.lhs) | rhs.zero_extended::<u32>(),
			Kind::Xor(rhs) => state.load_reg(self.lhs) ^ rhs.zero_extended::<u32>(),
		};
		state.store_reg(self.dst, value);

		Ok(())
	}
}
