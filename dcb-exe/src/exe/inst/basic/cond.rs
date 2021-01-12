//! Condition branches

// Imports
use crate::{
	exe::inst::{
		basic::{Decodable, Encodable},
		InstFmt, Register,
	},
	Pos,
};
use int_conv::{SignExtended, Signed, Truncated, ZeroExtended};
use std::fmt;

/// Raw representation
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Raw {
	/// Opcode (lower 3 bits)
	pub p: u32,

	/// Rs
	pub s: u32,

	/// Rt
	pub t: u32,

	/// Immediate
	pub i: u32,
}

/// Condition kind
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
	/// Returns the target for this instruction
	#[must_use]
	pub fn target(self, pos: Pos) -> Pos {
		Self::target_of(self.offset, pos)
	}

	/// Returns the target using an offset
	#[must_use]
	pub fn target_of(offset: i16, pos: Pos) -> Pos {
		pos + 4 * (offset.sign_extended::<i32>() + 1)
	}
}

impl Decodable for Inst {
	type Raw = Raw;

	fn decode(raw: Self::Raw) -> Option<Self> {
		let kind = match raw.p {
			0x1 => match raw.t {
				0b00000 => Kind::LessThanZero,
				0b00001 => Kind::GreaterOrEqualZero,
				0b10000 => Kind::LessThanZeroLink,
				0b10001 => Kind::GreaterOrEqualZeroLink,
				_ => return None,
			},
			0x4 => Kind::Equal(Register::new(raw.t)?),
			0x5 => Kind::NotEqual(Register::new(raw.t)?),
			0x6 => Kind::LessOrEqualZero,
			0x7 => Kind::GreaterThanZero,
			_ => return None,
		};

		Some(Self {
			arg: Register::new(raw.s)?,
			offset: raw.i.truncated::<u16>().as_signed(),
			kind,
		})
	}
}

impl Encodable for Inst {
	fn encode(&self) -> Self::Raw {
		#[rustfmt::skip]
		let (p, t) = match self.kind {
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
		let i = self.offset.as_unsigned().zero_extended();

		Raw { p, s, t, i }
	}
}

impl InstFmt for Inst {
	fn fmt(&self, pos: Pos, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", self.fmt_target(self.target(pos)))
	}
}

impl Inst {
	/// Returns a formattable for this instruction using `target` as it's target.
	pub fn fmt_target<'a>(self, target: impl fmt::Display + 'a) -> impl fmt::Display + 'a {
		dcb_util::DisplayWrapper::new(move |f| {
			let Self { kind, arg, .. } = self;

			// `beq $zr, $zr, offset` => `b offset`
			// `beq $zr, $arg, offset` => `beqz $arg, offset`
			// `beq $zr, $arg, offset` => `bnez $arg, offset`
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
		})
	}
}
