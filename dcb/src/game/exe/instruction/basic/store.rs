//! Store instructions

// Imports
use crate::game::exe::instruction::Register;
use int_conv::{Signed, Truncated, ZeroExtended};
use std::{convert::TryFrom, fmt};

/// Store instruction opcode (lower 3 bits)
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[derive(num_enum::IntoPrimitive, num_enum::TryFromPrimitive)]
#[repr(u8)]
pub enum StoreOpcode {
	/// Byte, `u8`
	Byte      = 0x0,

	/// Half-word, `u16`
	HalfWord  = 0x1,

	/// Word left-bits, `u32`
	WordLeft  = 0x2,

	/// Word, `u32`
	Word      = 0x3,

	/// Word right-bits, `u32`
	WordRight = 0x6,
}

/// Raw representation
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct StoreRaw {
	/// Opcode (lower 3 bits)
	pub p: u32,

	/// Rs
	pub s: u32,

	/// Rt
	pub t: u32,

	/// Immediate
	pub i: u32,
}

/// Store instructions
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct StoreInst {
	/// Source register, `rt`
	pub source: Register,

	/// Destination register, `rs`
	pub dest: Register,

	/// Destination offset.
	pub offset: i16,

	/// Opcode
	pub op: StoreOpcode,
}

impl StoreInst {
	/// Decodes this instruction
	#[must_use]
	pub fn decode(raw: StoreRaw) -> Option<Self> {
		let kind = StoreOpcode::try_from(raw.p.truncated::<u8>()).ok()?;

		Some(Self {
			source: Register::new(raw.t)?,
			dest:   Register::new(raw.s)?,
			offset: raw.i.truncated::<u16>().as_signed(),
			op:     kind,
		})
	}

	/// Encodes this instruction
	#[must_use]
	pub fn encode(self) -> StoreRaw {
		let t = self.source.idx();
		let s = self.dest.idx();
		let i = self.offset.as_unsigned().zero_extended::<u32>();
		let p = u8::from(self.op).zero_extended::<u32>();

		StoreRaw { p, s, t, i }
	}
}

impl fmt::Display for StoreInst {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let Self {
			source,
			dest,
			offset,
			op: kind,
		} = self;

		let mnemonic = match kind {
			StoreOpcode::Byte => "sb",
			StoreOpcode::HalfWord => "sh",
			StoreOpcode::Word => "sw",
			StoreOpcode::WordRight => "swr",
			StoreOpcode::WordLeft => "swl",
		};

		write!(f, "{mnemonic} {dest}, {offset}({source})")
	}
}
