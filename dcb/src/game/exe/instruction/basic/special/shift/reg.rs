//! Register shifts

// Imports
use crate::game::exe::instruction::Register;
use int_conv::{Truncated, ZeroExtended};
use std::{convert::TryFrom, fmt};

/// Shift register instruction func
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[derive(num_enum::IntoPrimitive, num_enum::TryFromPrimitive)]
#[repr(u8)]
pub enum ShiftRegFunc {
	/// Left logical
	LeftLogical     = 0x4,

	/// Right logical
	RightLogical    = 0x6,

	/// Right arithmetic
	RightArithmetic = 0x7,
}

/// Raw representation
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct ShiftRegRaw {
	/// Rs
	pub s: u32,

	/// Rt
	pub t: u32,

	/// Rd
	pub d: u32,

	/// Func
	pub f: u32,
}

/// Shift register instructions
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct ShiftRegInst {
	/// Destination register, `rd`
	pub dest: Register,

	/// Lhs argument, `rt`
	pub lhs: Register,

	/// Rhs argument, `rs`
	pub rhs: Register,

	/// Function
	pub func: ShiftRegFunc,
}

impl ShiftRegInst {
	/// Decodes this instruction
	#[must_use]
	pub fn decode(raw: ShiftRegRaw) -> Option<Self> {
		let func = ShiftRegFunc::try_from(raw.f.truncated::<u8>()).ok()?;

		Some(Self {
			dest: Register::new(raw.d)?,
			lhs: Register::new(raw.t)?,
			rhs: Register::new(raw.s)?,
			func,
		})
	}

	/// Encodes this instruction
	#[must_use]
	pub fn encode(self) -> ShiftRegRaw {
		let d = self.dest.idx();
		let t = self.lhs.idx();
		let s = self.rhs.idx();
		let f = u8::from(self.func).zero_extended::<u32>();

		ShiftRegRaw { f, t, d, s }
	}
}

impl fmt::Display for ShiftRegInst {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let Self { dest, lhs, rhs, func } = self;

		let mnemonic = match func {
			ShiftRegFunc::LeftLogical => "sllv",
			ShiftRegFunc::RightLogical => "srlv",
			ShiftRegFunc::RightArithmetic => "srav",
		};

		write!(f, "{mnemonic} {dest}, {lhs}, {rhs}")
	}
}
