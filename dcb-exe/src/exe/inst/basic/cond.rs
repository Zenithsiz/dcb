//! Condition branches

// Imports
use crate::exe::inst::{
	basic::{Decodable, Encodable},
	Register,
};
use dcb_util::SignedHex;
use int_conv::{Signed, Truncated, ZeroExtended};
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
	fn encode(&self) -> Raw {
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

// TODO: Fmt given `pc` / label

impl fmt::Display for Inst {
	#[rustfmt::skip]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let Self { arg, offset, kind } = self;

		match kind {
			Kind::Equal(reg)             => write!(f, "beq {arg}, {reg}, {}", SignedHex(offset)),
			Kind::NotEqual(reg)          => write!(f, "bne {arg}, {reg}, {}", SignedHex(offset)),
			Kind::LessOrEqualZero        => write!(f, "blez {arg}, {}"      , SignedHex(offset)),
			Kind::GreaterThanZero        => write!(f, "bgtz {arg}, {}"      , SignedHex(offset)),
			Kind::LessThanZero           => write!(f, "bltz {arg}, {}"      , SignedHex(offset)),
			Kind::GreaterOrEqualZero     => write!(f, "bgez {arg}, {}"      , SignedHex(offset)),
			Kind::LessThanZeroLink       => write!(f, "bltzal {arg}, {}"    , SignedHex(offset)),
			Kind::GreaterOrEqualZeroLink => write!(f, "bgezal {arg}, {}"    , SignedHex(offset)),
		}
	}
}