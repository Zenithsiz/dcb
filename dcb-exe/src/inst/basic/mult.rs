//! Multiplications

// Imports
use super::ModifiesReg;
use crate::inst::{
	basic::{Decode, Encode},
	exec::{ExecCtx, ExecError, Executable},
	parse::LineArg,
	DisplayCtx, InstDisplay, InstFmtArg, Parsable, ParseCtx, ParseError, Register,
};
use int_conv::{SignExtended, Signed, Split, ZeroExtended};
use std::array;

/// Operation kind
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum MultKind {
	/// Multiplication
	Mult,

	/// Division
	Div,
}

/// Operation mode
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum MultMode {
	/// Signed
	Signed,

	/// Unsigned
	Unsigned,
}

/// Multiplication register
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[derive(derive_more::Display)]
pub enum MultReg {
	/// Lo
	#[display(fmt = "$lo")]
	Lo,

	/// Hi
	#[display(fmt = "$hi")]
	Hi,
}

/// Multiplication instructions
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Inst {
	/// Multiplication
	Mult {
		/// Kind
		kind: MultKind,

		/// Mode
		mode: MultMode,

		/// Lhs argument
		lhs: Register,

		/// Rhs argument
		rhs: Register,
	},

	/// Move from
	MoveFrom {
		/// Destination
		dst: Register,

		/// Source
		src: MultReg,
	},

	/// Move to
	MoveTo {
		/// Source
		src: Register,

		/// Destination
		dst: MultReg,
	},
}

impl Inst {
	/// Returns this instruction's mnemonic
	#[must_use]
	const fn mnemonic(self) -> &'static str {
		match self {
			#[rustfmt::skip]
			Self::Mult { kind, mode, .. } => match (kind, mode) {
				(MultKind::Mult, MultMode::  Signed) => "mult",
				(MultKind::Mult, MultMode::Unsigned) => "multu",
				(MultKind::Div , MultMode::  Signed) => "div",
				(MultKind::Div , MultMode::Unsigned) => "divu",
			},
			Self::MoveFrom { src, .. } => match src {
				MultReg::Hi => "mfhi",
				MultReg::Lo => "mflo",
			},
			Self::MoveTo { dst, .. } => match dst {
				MultReg::Hi => "mthi",
				MultReg::Lo => "mtlo",
			},
		}
	}
}

impl Decode for Inst {
	#[rustfmt::skip]
	#[bitmatch::bitmatch]
	fn decode(raw: u32) -> Option<Self> {
		let [s, t, d, f] = #[bitmatch] match raw {
			"000000_sssss_ttttt_ddddd_?????_01ffff" => [s, t, d, f],
			_ => return None,
		};

		let reg = Register::new;
		Some(match f {
			// 00x0
			0x0 => Self::MoveFrom { dst: reg(d)?, src: MultReg::Hi },
			0x2 => Self::MoveFrom { dst: reg(d)?, src: MultReg::Lo },

			// 00x1
			0x1 => Self::MoveTo { src: reg(s)?, dst: MultReg::Hi },
			0x3 => Self::MoveTo { src: reg(s)?, dst: MultReg::Lo },

			// 10xx
			0x8 => Self::Mult { kind: MultKind::Mult, mode: MultMode::  Signed, lhs: reg(s)?, rhs: reg(t)? },
			0x9 => Self::Mult { kind: MultKind::Mult, mode: MultMode::Unsigned, lhs: reg(s)?, rhs: reg(t)? },
			0xa => Self::Mult { kind: MultKind::Div , mode: MultMode::  Signed, lhs: reg(s)?, rhs: reg(t)? },
			0xb => Self::Mult { kind: MultKind::Div , mode: MultMode::Unsigned, lhs: reg(s)?, rhs: reg(t)? },

			_ => return None,
		})
	}
}

impl Encode for Inst {
	#[rustfmt::skip]
	#[bitmatch::bitmatch]
	fn encode(&self) -> u32 {
		let [s, t, d, f] = match self {
			Self::Mult { kind, mode, lhs, rhs } => [lhs.idx(), rhs.idx(), 0, match (kind, mode) {
					(MultKind::Mult, MultMode::Signed  ) => 0x8,
					(MultKind::Mult, MultMode::Unsigned) => 0x9,
					(MultKind::Div , MultMode::Signed  ) => 0xa,
					(MultKind::Div , MultMode::Unsigned) => 0xb,
			}],
			Self::MoveFrom { dst, src } => [0, 0, dst.idx(), match src {
					MultReg::Hi => 0x0,
					MultReg::Lo => 0x2,
			}],
			Self::MoveTo { dst, src } => [src.idx(), 0, 0, match dst {
					MultReg::Hi => 0x1,
					MultReg::Lo => 0x3,
			}],
		};

		bitpack!("000000_sssss_ttttt_ddddd_?????_01ffff")
	}
}

impl<'a> Parsable<'a> for Inst {
	fn parse<Ctx: ?Sized + ParseCtx<'a>>(
		mnemonic: &'a str, args: &'a [LineArg], _ctx: &Ctx,
	) -> Result<Self, ParseError> {
		let inst = match mnemonic {
			"mflo" | "mfhi" | "mtlo" | "mthi" => {
				let reg = match *args {
					[LineArg::Register(reg)] => reg,
					_ => return Err(ParseError::InvalidArguments),
				};

				let mult_reg = match &mnemonic[2..=3] {
					"lo" => MultReg::Lo,
					"hi" => MultReg::Hi,
					_ => unreachable!(),
				};

				match &mnemonic[1..=1] {
					"f" => Inst::MoveFrom {
						dst: reg,
						src: mult_reg,
					},
					"t" => Inst::MoveTo {
						dst: mult_reg,
						src: reg,
					},
					_ => unreachable!(),
				}
			},

			// Mult / Div
			"mult" | "multu" | "div" | "divu" => {
				let (lhs, rhs) = match *args {
					[LineArg::Register(lhs), LineArg::Register(rhs)] => (lhs, rhs),
					_ => return Err(ParseError::InvalidArguments),
				};

				Inst::Mult {
					lhs,
					rhs,
					mode: match mnemonic {
						"divu" | "multu" => MultMode::Unsigned,
						"div" | "mult" => MultMode::Signed,
						_ => unreachable!(),
					},
					kind: match mnemonic {
						"mult" | "multu" => MultKind::Mult,
						"div" | "divu" => MultKind::Div,
						_ => unreachable!(),
					},
				}
			},

			_ => return Err(ParseError::UnknownMnemonic),
		};

		Ok(inst)
	}
}

impl<'a> InstDisplay<'a> for Inst {
	type Mnemonic = &'static str;

	type Args = impl Iterator<Item = InstFmtArg<'a>>;

	fn mnemonic<Ctx: DisplayCtx>(&'a self, _ctx: &Ctx) -> Self::Mnemonic {
		Self::mnemonic(*self)
	}

	#[auto_enums::auto_enum(Iterator)]
	fn args<Ctx: DisplayCtx>(&'a self, _ctx: &Ctx) -> Self::Args {
		match *self {
			Self::Mult { lhs, rhs, .. } => array::IntoIter::new([InstFmtArg::Register(lhs), InstFmtArg::Register(rhs)]),
			Self::MoveFrom { dst: arg, .. } | Self::MoveTo { src: arg, .. } => {
				array::IntoIter::new([InstFmtArg::Register(arg)])
			},
		}
	}
}

impl ModifiesReg for Inst {
	fn modifies_reg(&self, reg: Register) -> bool {
		match self {
			Inst::MoveFrom { dst, .. } => *dst == reg,
			Inst::Mult { .. } | Inst::MoveTo { .. } => false,
		}
	}
}

impl Executable for Inst {
	fn exec<Ctx: ExecCtx>(&self, state: &mut Ctx) -> Result<(), ExecError> {
		match *self {
			Inst::Mult { kind, mode, lhs, rhs } => {
				let (lo, hi) = match (kind, mode) {
					(MultKind::Mult, MultMode::Signed) => {
						let lhs: i64 = state.load_reg(lhs).as_signed().sign_extended();
						let rhs: i64 = state.load_reg(rhs).as_signed().sign_extended();

						lhs.wrapping_mul(rhs).as_unsigned().lo_hi()
					},
					(MultKind::Mult, MultMode::Unsigned) => {
						let lhs: u64 = state.load_reg(lhs).zero_extended();
						let rhs: u64 = state.load_reg(rhs).zero_extended();

						lhs.wrapping_mul(rhs).as_unsigned().lo_hi()
					},
					(MultKind::Div, MultMode::Signed) => {
						match (state.load_reg(lhs).as_signed(), state.load_reg(rhs).as_signed()) {
							(lhs @ 0i32..=i32::MAX, 0i32) => ((-1i32).as_unsigned(), lhs.as_unsigned()),
							(lhs @ i32::MIN..0i32, 0i32) => (1u32, lhs.as_unsigned()),
							(lhs, rhs) => (lhs.wrapping_div(rhs).as_unsigned(), lhs.wrapping_rem(rhs).as_unsigned()),
						}
					},
					(MultKind::Div, MultMode::Unsigned) => match (state.load_reg(lhs), state.load_reg(rhs)) {
						(lhs, 0) => ((-1i32).as_unsigned(), lhs),
						(lhs, rhs) => (lhs / rhs, lhs % rhs),
					},
				};

				state.store_mult_reg(MultReg::Lo, lo);
				state.store_mult_reg(MultReg::Hi, hi);

				Ok(())
			},
			Inst::MoveFrom { dst, src } => {
				state.store_reg(dst, state.load_mult_reg(src));
				Ok(())
			},
			Inst::MoveTo { src, dst } => {
				state.store_mult_reg(dst, state.load_reg(src));
				Ok(())
			},
		}
	}
}
