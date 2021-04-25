//! Condition branches

// Imports
use super::ModifiesReg;
use crate::{
	inst::{
		basic::{Decodable, Encodable},
		InstTarget, InstTargetFmt, Register,
	},
	Pos,
};
use int_conv::{SignExtended, Signed, Truncated, ZeroExtended};
use std::fmt;

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

impl Decodable for Inst {
	type Raw = u32;

	#[bitmatch::bitmatch]
	fn decode(raw: Self::Raw) -> Option<Self> {
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

impl Encodable for Inst {
	#[bitmatch::bitmatch]
	fn encode(&self) -> Self::Raw {
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

impl InstTarget for Inst {
	fn target(&self, pos: Pos) -> Pos {
		Self::target_of(self.offset, pos)
	}
}

impl InstTargetFmt for Inst {
	fn fmt(&self, _pos: Pos, target: impl fmt::Display, f: &mut fmt::Formatter) -> fmt::Result {
		let Self { kind, arg, .. } = self;

		// `beq $zr, $zr, offset` => `b offset`
		// `beq $zr, $arg, offset` => `beqz $arg, offset`
		// `bne $zr, $arg, offset` => `bnez $arg, offset`
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
