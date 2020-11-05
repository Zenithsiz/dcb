//! Immediate shifts

// Imports
use crate::game::exe::instruction::Register;
use int_conv::{Signed, Truncated, ZeroExtended};
use std::{convert::TryFrom, fmt};

/// Shift immediate instruction func
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[derive(num_enum::IntoPrimitive, num_enum::TryFromPrimitive)]
#[repr(u8)]
pub enum ShiftImmFunc {
	/// Left logical
	LeftLogical     = 0x0,

	/// Right logical
	RightLogical    = 0x2,

	/// Right arithmetic
	RightArithmetic = 0x3,
}

/// Raw representation
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct ShiftImmRaw {
	/// Rt
	pub t: u32,

	/// Rd
	pub d: u32,

	/// Immediate
	pub i: u32,

	/// Func
	pub f: u32,
}

/// Shift immediate instructions
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct ShiftImmInst {
	/// Destination register, `rt`
	pub dst: Register,

	/// Lhs argument, `rd`
	pub lhs: Register,

	/// Rhs argument, immediate
	pub rhs: i16,

	/// Function
	pub func: ShiftImmFunc,
}

impl ShiftImmInst {
	/// Decodes this instruction
	#[must_use]
	pub fn decode(raw: ShiftImmRaw) -> Option<Self> {
		let func = ShiftImmFunc::try_from(raw.f.truncated::<u8>()).ok()?;

		Some(Self {
			lhs: Register::new(raw.t)?,
			dst: Register::new(raw.d)?,
			rhs: raw.i.truncated::<u16>().as_signed(),
			func,
		})
	}

	/// Encodes this instruction
	#[must_use]
	pub fn encode(self) -> ShiftImmRaw {
		let t = self.lhs.idx();
		let d = self.dst.idx();
		let i = self.rhs.as_unsigned().zero_extended::<u32>();
		let f = u8::from(self.func).zero_extended::<u32>();

		ShiftImmRaw { f, t, d, i }
	}
}

impl fmt::Display for ShiftImmInst {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let Self { lhs, dst, rhs, func } = self;

		let mnemonic = match func {
			ShiftImmFunc::LeftLogical => "sll",
			ShiftImmFunc::RightLogical => "srl",
			ShiftImmFunc::RightArithmetic => "sra",
		};

		write!(f, "{mnemonic} {dst}, {lhs}, {rhs:#x}")
	}
}
