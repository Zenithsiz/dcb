//! Alu register instructions

// Imports
use crate::inst::{
	basic::{Decode, Encode, ModifiesReg},
	exec::{ExecCtx, ExecError, Executable},
	parse::LineArg,
	DisplayCtx, InstDisplay, InstFmtArg, Parsable, ParseCtx, ParseError, Register,
};
use int_conv::Signed;
use std::array;

/// Alu register instruction kind
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Kind {
	/// Add signed with overflow trap
	Add,

	/// Add signed without overflow trap
	AddUnsigned,

	/// Sub signed with overflow trap
	Sub,

	/// Sub signed without overflow trap
	SubUnsigned,

	/// Bit and
	And,

	/// Bit or
	Or,

	/// Bit xor
	Xor,

	/// Bit nor
	Nor,

	/// Set on less than signed
	SetLessThan,

	/// Set on less than unsigned
	SetLessThanUnsigned,
}

impl Kind {
	/// Returns this kind's mnemonic
	#[must_use]
	pub const fn mnemonic(self) -> &'static str {
		match self {
			Self::Add => "add",
			Self::AddUnsigned => "addu",
			Self::Sub => "sub",
			Self::SubUnsigned => "subu",
			Self::And => "and",
			Self::Or => "or",
			Self::Xor => "xor",
			Self::Nor => "nor",
			Self::SetLessThan => "slt",
			Self::SetLessThanUnsigned => "sltu",
		}
	}
}

/// Alu register instructions
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
			"000000_sssss_ttttt_ddddd_?????_10ffff" => [s, t, d, f],
			_ => return None,
		};

		Some(Self {
			dst:  Register::new(d)?,
			lhs:  Register::new(s)?,
			rhs:  Register::new(t)?,
			kind: match f {
				0x0 => Kind::Add,
				0x1 => Kind::AddUnsigned,
				0x2 => Kind::Sub,
				0x3 => Kind::SubUnsigned,
				0x4 => Kind::And,
				0x5 => Kind::Or,
				0x6 => Kind::Xor,
				0x7 => Kind::Nor,
				0xa => Kind::SetLessThan,
				0xb => Kind::SetLessThanUnsigned,
				_ => return None,
			},
		})
	}
}

impl Encode for Inst {
	#[bitmatch::bitmatch]
	fn encode(&self) -> u32 {
		#[rustfmt::skip]
		let f: u32 = match self.kind {
			Kind::Add                 => 0x0,
			Kind::AddUnsigned         => 0x1,
			Kind::Sub                 => 0x2,
			Kind::SubUnsigned         => 0x3,
			Kind::And                 => 0x4,
			Kind::Or                  => 0x5,
			Kind::Xor                 => 0x6,
			Kind::Nor                 => 0x7,
			Kind::SetLessThan         => 0xa,
			Kind::SetLessThanUnsigned => 0xb,
		};

		let d = self.dst.idx();
		let s = self.lhs.idx();
		let t = self.rhs.idx();

		bitpack!("000000_sssss_ttttt_ddddd_?????_10ffff")
	}
}

impl<'a> Parsable<'a> for Inst {
	fn parse<Ctx: ?Sized + ParseCtx<'a>>(
		mnemonic: &'a str, args: &'a [LineArg], _ctx: &Ctx,
	) -> Result<Self, ParseError> {
		#[rustfmt::skip]
		let kind = match mnemonic {
			"add"  => Kind::Add                ,
			"addu" => Kind::AddUnsigned        ,
			"sub"  => Kind::Sub                ,
			"subu" => Kind::SubUnsigned        ,
			"and"  => Kind::And                ,
			"or"   => Kind::Or                 ,
			"xor"  => Kind::Xor                ,
			"nor"  => Kind::Nor                ,
			"slt"  => Kind::SetLessThan        ,
			"sltu" => Kind::SetLessThanUnsigned,
			_ => return Err(ParseError::UnknownMnemonic),
		};

		let inst = match *args {
			// Disallow `slt` and `sltu` in short form
			[LineArg::Register(_), LineArg::Register(_)] if ["slt", "sltu"].contains(&mnemonic) => {
				return Err(ParseError::InvalidArguments)
			},

			// Else parse both `$dst, $lhs, $rhs` and `$dst, $rhs`.
			[LineArg::Register(lhs @ dst), LineArg::Register(rhs)] |
			[LineArg::Register(dst), LineArg::Register(lhs), LineArg::Register(rhs)] => Self { dst, lhs, rhs, kind },
			_ => return Err(ParseError::InvalidArguments),
		};

		Ok(inst)
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
		let &Self { dst, lhs, rhs, kind } = self;

		// If we're not `slt[u]` and if `$dst` and `$lhs` are the same,
		// only return one of them
		match !matches!(kind, Kind::SetLessThan | Kind::SetLessThanUnsigned) && dst == lhs {
			true => array::IntoIter::new([InstFmtArg::Register(dst), InstFmtArg::Register(rhs)]),
			false => array::IntoIter::new([
				InstFmtArg::Register(dst),
				InstFmtArg::Register(lhs),
				InstFmtArg::Register(rhs),
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
		let lhs = state[self.lhs];
		let rhs = state[self.rhs];

		state[self.dst] = match self.kind {
			Kind::Add => lhs
				.as_signed()
				.checked_add(rhs.as_signed())
				.ok_or(ExecError::Overflow)?
				.as_unsigned(),
			Kind::AddUnsigned => lhs.as_signed().wrapping_add(rhs.as_signed()).as_unsigned(),
			Kind::Sub => lhs
				.as_signed()
				.checked_sub(rhs.as_signed())
				.ok_or(ExecError::Overflow)?
				.as_unsigned(),
			Kind::SubUnsigned => lhs.as_signed().wrapping_sub(rhs.as_signed()).as_unsigned(),
			Kind::And => lhs & rhs,
			Kind::Or => lhs | rhs,
			Kind::Xor => lhs ^ rhs,
			Kind::Nor => !(lhs | rhs),
			Kind::SetLessThan => (lhs.as_signed() < rhs.as_signed()).into(),
			Kind::SetLessThanUnsigned => (lhs < rhs).into(),
		};

		Ok(())
	}
}
