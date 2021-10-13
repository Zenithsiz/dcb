//! Load immediate

// Imports
use super::{Decodable, Encodable};
use crate::{
	inst::{
		basic, parse::LineArg, DisplayCtx, InstDisplay, InstFmtArg, InstSize, Parsable, ParseCtx, ParseError, Register,
	},
	Pos,
};
use int_conv::{Join, SignExtended, Signed, Split};
use std::convert::TryInto;

/// Immediate kind
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[derive(derive_more::Display)]
pub enum Kind {
	/// Address
	///
	/// Alias for `lui $dst, {hi} / addiu $dst, $dst, {lo}`
	Address(Pos),

	/// Word
	///
	/// Alias for `lui $dst, {hi} / ori $dst, $dst, {lo}`
	Word(u32),

	/// Unsigned half-word
	///
	/// Alias for `ori $dst, $zr, imm`
	HalfWordUnsigned(u16),

	/// Signed half-word
	///
	/// Alias for `addiu $dst, $zr, imm`
	HalfWordSigned(i16),
}

impl Kind {
	/// Returns the mnemonic for this load kind
	#[must_use]
	pub const fn mnemonic(self) -> &'static str {
		match self {
			Self::Address(_) => "la",
			Self::Word(_) | Self::HalfWordUnsigned(_) | Self::HalfWordSigned(_) => "li",
		}
	}
}

/// Load immediate instruction
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Inst {
	/// Destination register
	pub dst: Register,

	/// Load kind
	pub kind: Kind,
}

impl Decodable for Inst {
	fn decode(mut insts: impl Iterator<Item = basic::Inst> + Clone) -> Option<Self> {
		use basic::alu::imm::Kind::{AddUnsigned, Or};
		let inst = match insts.next()? {
			// `lui $dst, $value`
			basic::Inst::Lui(lui) => match insts.next()?.try_into().ok()? {
				// Filter for same `$dst` and equal `$dst` and `$lhs`.
				basic::alu::Inst::Imm(alu) if lui.dst == alu.dst && alu.dst == alu.lhs => Self {
					dst:  lui.dst,
					kind: match alu.kind {
						// lui << 16 + rhs
						AddUnsigned(rhs) => Kind::Address(Pos((u32::join(0, lui.value).as_signed() +
							rhs.sign_extended::<i32>())
						.as_unsigned())),
						Or(rhs) => Kind::Word(u32::join(rhs, lui.value)),
						_ => return None,
					},
				},
				_ => return None,
			},
			// `addiu $zr, $value`
			// `ori   $zr, $value`
			#[rustfmt::skip]
			basic::Inst::Alu(basic::alu::Inst::Imm(inst)) if inst.lhs == Register::Zr => Self {
				dst:        inst.dst,
				kind: match inst.kind {
					AddUnsigned(value) => Kind::HalfWordSigned  (value),
					Or         (value) => Kind::HalfWordUnsigned(value),
					_ => return None,
				},
			},

			_ => return None,
		};

		Some(inst)
	}
}

impl<'a> Encodable<'a> for Inst {
	type Iterator = impl Iterator<Item = basic::Inst> + 'a;

	#[auto_enums::auto_enum(Iterator)]
	fn encode(&'a self) -> Self::Iterator {
		match self.kind {
			Kind::Address(Pos(addr)) => {
				let (lo, hi) = match addr.lo().as_signed() < 0 {
					true => (addr.lo(), addr.hi().wrapping_add(1)),
					false => addr.lo_hi(),
				};

				[
					basic::Inst::Lui(basic::lui::Inst {
						dst:   self.dst,
						value: hi,
					}),
					basic::Inst::Alu(basic::alu::Inst::Imm(basic::alu::imm::Inst {
						dst:  self.dst,
						lhs:  self.dst,
						kind: basic::alu::imm::Kind::AddUnsigned(lo.as_signed()),
					})),
				]
				.into_iter()
			},
			Kind::Word(value) => {
				let (lo, hi) = value.lo_hi();

				[
					basic::Inst::Lui(basic::lui::Inst {
						dst:   self.dst,
						value: hi,
					}),
					basic::Inst::Alu(basic::alu::Inst::Imm(basic::alu::imm::Inst {
						dst:  self.dst,
						lhs:  self.dst,
						kind: basic::alu::imm::Kind::Or(lo),
					})),
				]
				.into_iter()
			},
			Kind::HalfWordUnsigned(value) => {
				std::iter::once(basic::Inst::Alu(basic::alu::Inst::Imm(basic::alu::imm::Inst {
					dst:  self.dst,
					lhs:  Register::Zr,
					kind: basic::alu::imm::Kind::Or(value),
				})))
			},

			Kind::HalfWordSigned(value) => {
				std::iter::once(basic::Inst::Alu(basic::alu::Inst::Imm(basic::alu::imm::Inst {
					dst:  self.dst,
					lhs:  Register::Zr,
					kind: basic::alu::imm::Kind::AddUnsigned(value),
				})))
			},
		}
	}
}

impl<'a> Parsable<'a> for Inst {
	fn parse<Ctx: ?Sized + ParseCtx<'a>>(
		mnemonic: &'a str, args: &'a [LineArg], ctx: &Ctx,
	) -> Result<Self, ParseError> {
		let to_kind = match mnemonic {
			"li" => |ctx: &Ctx, arg: &LineArg| match arg {
				// Try `i16`, `u16` then `u32` for the literal
				#[allow(clippy::same_functions_in_if_condition)] // Each one has a different type
				LineArg::Expr(expr) => {
					let value = ctx.eval_expr(expr)?;
					if let Ok(value) = value.try_into() {
						Ok(Kind::HalfWordSigned(value))
					} else if let Ok(value) = value.try_into() {
						Ok(Kind::HalfWordUnsigned(value))
					} else if let Ok(value) = value.try_into() {
						Ok(Kind::Word(value))
					} else {
						Err(ParseError::LiteralOutOfRange)
					}
				},
				_ => Err(ParseError::InvalidArguments),
			},
			"la" => |ctx: &Ctx, arg: &LineArg| ctx.arg_pos(arg).map(Kind::Address),
			_ => return Err(ParseError::UnknownMnemonic),
		};

		let (dst, kind) = match *args {
			[LineArg::Register(dst), ref arg] => (dst, to_kind(ctx, arg)?),
			_ => return Err(ParseError::InvalidArguments),
		};

		Ok(Self { dst, kind })
	}
}

impl<'a> InstDisplay<'a> for Inst {
	type Args = [InstFmtArg<'a>; 2];
	type Mnemonic = &'static str;

	fn mnemonic<Ctx: DisplayCtx>(&'a self, _ctx: &Ctx) -> Self::Mnemonic {
		self.kind.mnemonic()
	}

	fn args<Ctx: DisplayCtx>(&'a self, _ctx: &Ctx) -> Self::Args {
		let &Self { dst, kind } = self;

		let arg = match kind {
			Kind::Address(pos) => InstFmtArg::Target(pos),
			Kind::Word(value) => InstFmtArg::literal(value),
			Kind::HalfWordUnsigned(value) => InstFmtArg::literal(value),
			Kind::HalfWordSigned(value) => InstFmtArg::literal(value),
		};
		[InstFmtArg::Register(dst), arg]
	}
}

impl InstSize for Inst {
	fn size(&self) -> usize {
		match self.kind {
			Kind::Address(_) | Kind::Word(_) => 8,
			Kind::HalfWordUnsigned(_) | Kind::HalfWordSigned(_) => 4,
		}
	}
}
