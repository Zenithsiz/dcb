//! Condition branches

// Imports
use super::ModifiesReg;
use crate::{
	inst::{
		basic::{Decode, Encode},
		parse::LineArg,
		DisplayCtx, InstDisplay, InstFmtArg, InstTarget, InstTargetFmt, Parsable, ParseCtx, ParseError, Register,
	},
	Pos,
};
use int_conv::{SignExtended, Signed, Truncated, ZeroExtended};
use std::{array, convert::TryInto, fmt};

/// Instruction kind
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Kind {
	/// Equal
	Equal(Register),

	/// Not equal
	NotEqual(Register),

	/// Less than or zero
	LessOrEqualZero,

	/// Greater than zero
	GreaterThanZero,

	/// Less than zero
	LessThanZero,

	/// Greater than or zero
	GreaterOrEqualZero,

	/// Less than zero and link
	LessThanZeroLink,

	/// Greater than or zero and link
	GreaterOrEqualZeroLink,
}

/// Condition instructions
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Inst {
	/// Argument, `rs`
	pub arg: Register,

	/// Offset
	pub offset: i16,

	/// Kind
	pub kind: Kind,
}

impl Inst {
	/// Returns the target using an offset
	#[must_use]
	pub fn target_of(offset: i16, pos: Pos) -> Pos {
		pos + 4i32 * (offset.sign_extended::<i32>() + 1i32)
	}
}

impl Decode for Inst {
	#[bitmatch::bitmatch]
	fn decode(raw: u32) -> Option<Self> {
		let [p, s, t, i] = #[bitmatch]
		match raw {
			"000ppp_sssss_ttttt_iiiii_iiiii_iiiiii" => [p, s, t, i],
			_ => return None,
		};

		Some(Self {
			arg:    Register::new(s)?,
			offset: i.truncated::<u16>().as_signed(),
			kind:   match p {
				0x1 => match t {
					0b00000 => Kind::LessThanZero,
					0b00001 => Kind::GreaterOrEqualZero,
					0b10000 => Kind::LessThanZeroLink,
					0b10001 => Kind::GreaterOrEqualZeroLink,
					_ => return None,
				},
				0x4 => Kind::Equal(Register::new(t)?),
				0x5 => Kind::NotEqual(Register::new(t)?),
				0x6 => Kind::LessOrEqualZero,
				0x7 => Kind::GreaterThanZero,
				_ => return None,
			},
		})
	}
}

impl Encode for Inst {
	#[bitmatch::bitmatch]
	fn encode(&self) -> u32 {
		#[rustfmt::skip]
		let (p, t): (u32, u32) = match self.kind {
			Kind::Equal(reg)             => (0x4, reg.idx()),
			Kind::NotEqual(reg)          => (0x5, reg.idx()),
			Kind::LessOrEqualZero        => (0x6, 0),
			Kind::GreaterThanZero        => (0x7, 0),
			Kind::LessThanZero           => (0x1, 0b00000),
			Kind::GreaterOrEqualZero     => (0x1, 0b00001),
			Kind::LessThanZeroLink       => (0x1, 0b10000),
			Kind::GreaterOrEqualZeroLink => (0x1, 0b10001),
		};

		let s = self.arg.idx();
		let i: u32 = self.offset.as_unsigned().zero_extended();

		bitpack!("000ppp_sssss_ttttt_iiiii_iiiii_iiiiii")
	}
}

impl Parsable for Inst {
	fn parse<Ctx: ?Sized + ParseCtx>(mnemonic: &str, args: &[LineArg], ctx: &Ctx) -> Result<Self, ParseError> {
		// Note: Literals are absolute, not relative

		// Calculates the offset between a position and the current one
		// with a possible offset
		let offset_of = |pos: Pos, offset: i64| -> Result<i16, ParseError> {
			use std::ops::{Add, Div, Sub};
			pos.sub(ctx.cur_pos())
				.add(offset)
				.div(4)
				.sub(1)
				.try_into()
				.map_err(|_| ParseError::RelativeJumpTooFar)
		};

		// Calculates the offset of a literal/label/label offset argument
		let target_arg_to_offset = |arg| ctx.arg_pos_offset(arg).and_then(|(pos, offset)| offset_of(pos, offset));

		let (arg, offset, kind) = match mnemonic {
			"b" => match args {
				[target] => (Register::Zr, target_arg_to_offset(target)?, Kind::Equal(Register::Zr)),
				_ => return Err(ParseError::InvalidArguments),
			},
			"beqz" => match *args {
				[LineArg::Register(arg), ref target] => (arg, target_arg_to_offset(target)?, Kind::Equal(Register::Zr)),
				_ => return Err(ParseError::InvalidArguments),
			},
			"bnez" => match *args {
				[LineArg::Register(arg), ref target] => (arg, target_arg_to_offset(target)?, Kind::NotEqual(Register::Zr)),
				_ => return Err(ParseError::InvalidArguments),
			},
			"beq" => match *args {
				[LineArg::Register(arg), LineArg::Register(reg), ref target] => (arg, target_arg_to_offset(target)?, Kind::Equal(reg)),
				_ => return Err(ParseError::InvalidArguments),
			},
			"bne" => match *args {
				[LineArg::Register(arg), LineArg::Register(reg), ref target] => (arg, target_arg_to_offset(target)?, Kind::NotEqual(reg)),
				_ => return Err(ParseError::InvalidArguments),
			},
			"blez" => match *args {
				[LineArg::Register(arg), ref target] => (arg, target_arg_to_offset(target)?, Kind::LessOrEqualZero),
				_ => return Err(ParseError::InvalidArguments),
			},
			"bgtz" => match *args {
				[LineArg::Register(arg), ref target] => (arg, target_arg_to_offset(target)?, Kind::GreaterThanZero),
				_ => return Err(ParseError::InvalidArguments),
			},
			"bltz" => match *args {
				[LineArg::Register(arg), ref target] => (arg, target_arg_to_offset(target)?, Kind::LessThanZero),
				_ => return Err(ParseError::InvalidArguments),
			},
			"bgez" => match *args {
				[LineArg::Register(arg), ref target] => (arg, target_arg_to_offset(target)?, Kind::GreaterOrEqualZero),
				_ => return Err(ParseError::InvalidArguments),
			},
			"bltzal" => match *args {
				[LineArg::Register(arg), ref target] => (arg, target_arg_to_offset(target)?, Kind::LessThanZeroLink),
				_ => return Err(ParseError::InvalidArguments),
			},
			"bgezal" => match *args {
				[LineArg::Register(arg), ref target] => (arg, target_arg_to_offset(target)?, Kind::GreaterOrEqualZeroLink),
				_ => return Err(ParseError::InvalidArguments),
			},

			_ => return Err(ParseError::UnknownMnemonic),
		};

		Ok(Self { arg, offset, kind })
	}
}

/// Variants:
/// `beq $zr, $zr, offset` => `b offset`
/// `beq $arg, $zr, offset` => `beqz $arg, offset`
/// `bne $arg, $zr, offset` => `bnez $arg, offset`
impl InstDisplay for Inst {
	type Mnemonic = &'static str;

	type Args = impl Iterator<Item = InstFmtArg>;

	fn mnemonic<Ctx: DisplayCtx>(&self, _ctx: &Ctx) -> Self::Mnemonic {
		match self.kind {
			Kind::Equal(Register::Zr) => match self.arg {
				Register::Zr => "b",
				_ => "beqz",
			},
			Kind::Equal(_) => "beq",
			Kind::NotEqual(Register::Zr) => "bnez",
			Kind::NotEqual(_) => "bne",
			Kind::LessOrEqualZero => "blez",
			Kind::GreaterThanZero => "bgtz",
			Kind::LessThanZero => "bltz",
			Kind::GreaterOrEqualZero => "bgez",
			Kind::LessThanZeroLink => "bltzal",
			Kind::GreaterOrEqualZeroLink => "bgezal",
		}
	}

	#[auto_enums::auto_enum(Iterator)]
	#[rustfmt::skip]
	fn args<Ctx: DisplayCtx>(&self, ctx: &Ctx) -> Self::Args {
		use InstFmtArg::Register as register;
		let &Self { arg, offset, kind } = self;
		let target = Self::target_of(offset, ctx.cur_pos());

		match (arg, kind) {
			(Register::Zr, Kind::Equal   (Register::Zr)) => array::IntoIter::new([                              InstFmtArg::Target(target)]),
			(_           , Kind::Equal   (Register::Zr)) => array::IntoIter::new([register(arg),                InstFmtArg::Target(target)]),
			(_           , Kind::Equal   (reg)         ) => array::IntoIter::new([register(arg), register(reg), InstFmtArg::Target(target)]),
			(_           , Kind::NotEqual(Register::Zr)) => array::IntoIter::new([register(arg),                InstFmtArg::Target(target)]),
			(_           , Kind::NotEqual(reg)         ) => array::IntoIter::new([register(arg), register(reg), InstFmtArg::Target(target)]),
			(
				_,
				Kind::LessOrEqualZero |
				Kind::GreaterThanZero |
				Kind::LessThanZero |
				Kind::GreaterOrEqualZero |
				Kind::LessThanZeroLink |
				Kind::GreaterOrEqualZeroLink,
			) => array::IntoIter::new([register(arg), InstFmtArg::Target(target)]),
		}
	}
}

impl InstTarget for Inst {
	fn target(&self, pos: Pos) -> Pos {
		Self::target_of(self.offset, pos)
	}
}

impl InstTargetFmt for Inst {
	fn fmt(&self, _pos: Pos, target: impl fmt::Display, f: &mut fmt::Formatter) -> fmt::Result {
		let Self { kind, arg, .. } = self;

		// `beq $zr, $zr, offset` => `b offset`
		// `beq $arg, $zr, offset` => `beqz $arg, offset`
		// `bne $arg, $zr, offset` => `bnez $arg, offset`
		match kind {
			Kind::Equal(Register::Zr) => match arg {
				Register::Zr => write!(f, "b {target}"),
				arg => write!(f, "beqz {arg}, {target}"),
			},
			Kind::Equal(reg) => write!(f, "beq {arg}, {reg}, {target}"),
			Kind::NotEqual(Register::Zr) => write!(f, "bnez {arg}, {target}"),
			Kind::NotEqual(reg) => write!(f, "bne {arg}, {reg}, {target}"),
			Kind::LessOrEqualZero => write!(f, "blez {arg}, {target}"),
			Kind::GreaterThanZero => write!(f, "bgtz {arg}, {target}"),
			Kind::LessThanZero => write!(f, "bltz {arg}, {target}"),
			Kind::GreaterOrEqualZero => write!(f, "bgez {arg}, {target}"),
			Kind::LessThanZeroLink => write!(f, "bltzal {arg}, {target}"),
			Kind::GreaterOrEqualZeroLink => write!(f, "bgezal {arg}, {target}"),
		}
	}
}

impl ModifiesReg for Inst {
	fn modifies_reg(&self, _reg: Register) -> bool {
		false
	}
}
