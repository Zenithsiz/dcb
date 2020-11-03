//! Condition branches

// Imports
use crate::{game::exe::instruction::Register, util::SignedHex};
use int_conv::{Signed, Truncated, ZeroExtended};
use std::fmt;

/// Raw representation
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct CondRaw {
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
pub enum CondKind {
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
pub struct CondInst {
	/// Argument, `rs`
	pub arg: Register,

	/// Offset
	pub offset: i16,

	/// Kind
	pub kind: CondKind,
}

impl CondInst {
	/// Decodes this instruction
	#[must_use]
	pub fn decode(raw: CondRaw) -> Option<Self> {
		let kind = match raw.p {
			0x1 => match raw.t {
				0b00000 => CondKind::LessThanZero,
				0b00001 => CondKind::GreaterOrEqualZero,
				0b10000 => CondKind::LessThanZeroLink,
				0b10001 => CondKind::GreaterOrEqualZeroLink,
				_ => return None,
			},
			0x4 => CondKind::Equal(Register::new(raw.t)?),
			0x5 => CondKind::NotEqual(Register::new(raw.t)?),
			0x6 => CondKind::LessOrEqualZero,
			0x7 => CondKind::GreaterThanZero,
			_ => return None,
		};

		Some(Self {
			arg: Register::new(raw.s)?,
			offset: raw.i.truncated::<u16>().as_signed(),
			kind,
		})
	}

	/// Encodes this instruction
	#[must_use]
	pub fn encode(self) -> CondRaw {
		#[rustfmt::skip]
		let (p, t) = match self.kind {
			CondKind::Equal(reg)             => (0x4, reg.idx()),
			CondKind::NotEqual(reg)          => (0x5, reg.idx()),
			CondKind::LessOrEqualZero        => (0x6, 0),
			CondKind::GreaterThanZero        => (0x7, 0),
			CondKind::LessThanZero           => (0x1, 0b00000),
			CondKind::GreaterOrEqualZero     => (0x1, 0b00001),
			CondKind::LessThanZeroLink       => (0x1, 0b10000),
			CondKind::GreaterOrEqualZeroLink => (0x1, 0b10001),
		};

		let s = self.arg.idx();
		let i = self.offset.as_unsigned().zero_extended();

		CondRaw { p, s, t, i }
	}
}

// TODO: Fmt given `pc` / label

impl fmt::Display for CondInst {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let Self { arg, offset, kind } = self;

		#[rustfmt::skip]
		match kind {
			CondKind::Equal(reg)             => write!(f, "beq {arg}, {reg}, {:#x}", SignedHex(offset)),
			CondKind::NotEqual(reg)          => write!(f, "bne {arg}, {reg}, {:#x}", SignedHex(offset)),
			CondKind::LessOrEqualZero        => write!(f, "blez {arg}, {:#x}"      , SignedHex(offset)),
			CondKind::GreaterThanZero        => write!(f, "bgtz {arg}, {:#x}"      , SignedHex(offset)),
			CondKind::LessThanZero           => write!(f, "bltz {arg}, {:#x}"      , SignedHex(offset)),
			CondKind::GreaterOrEqualZero     => write!(f, "bgez {arg}, {:#x}"      , SignedHex(offset)),
			CondKind::LessThanZeroLink       => write!(f, "bltzal {arg}, {:#x}"    , SignedHex(offset)),
			CondKind::GreaterOrEqualZeroLink => write!(f, "bgezal {arg}, {:#x}"    , SignedHex(offset)),
		}
	}
}
