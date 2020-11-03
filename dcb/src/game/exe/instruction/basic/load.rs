//! Load instructions

// Imports
use crate::game::exe::instruction::Register;
use int_conv::{Signed, Truncated, ZeroExtended};
use std::{convert::TryFrom, fmt};

/// Load instruction opcode (lower 3 bits)
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[derive(num_enum::IntoPrimitive, num_enum::TryFromPrimitive)]
#[repr(u8)]
pub enum LoadOpcode {
	/// Byte, `i8`
	Byte             = 0x0,

	/// Half-word, `i16`
	HalfWord         = 0x1,

	/// Word left-bits, `u32`
	WordLeft         = 0x2,

	/// Word, `u32`
	Word             = 0x3,

	/// Byte unsigned, `u8`
	ByteUnsigned     = 0x4,

	/// Half-word unsigned, `u16`
	HalfWordUnsigned = 0x5,

	/// Word right-bits, `u32`
	WordRight        = 0x6,
}

/// Raw representation
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct LoadRaw {
	/// Opcode (lower 3 bits)
	pub p: u32,

	/// Rs
	pub s: u32,

	/// Rt
	pub t: u32,

	/// Immediate
	pub i: u32,
}

/// Load instructions
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct LoadInst {
	/// Source register, `rt`
	pub source: Register,

	/// Destination register, `rs`
	pub dest: Register,

	/// Destination offset.
	pub offset: i16,

	/// Opcode
	pub op: LoadOpcode,
}

impl LoadInst {
	/// Decodes this instruction
	#[must_use]
	pub fn decode(raw: LoadRaw) -> Option<Self> {
		let op = LoadOpcode::try_from(raw.p.truncated::<u8>()).ok()?;

		Some(Self {
			source: Register::new(raw.t)?,
			dest: Register::new(raw.s)?,
			offset: raw.i.truncated::<u16>().as_signed(),
			op,
		})
	}

	/// Encodes this instruction
	#[must_use]
	pub fn encode(self) -> LoadRaw {
		let t = self.source.idx();
		let s = self.dest.idx();
		let i = self.offset.as_unsigned().zero_extended::<u32>();
		let p = u8::from(self.op).zero_extended::<u32>();

		LoadRaw { p, s, t, i }
	}
}

impl fmt::Display for LoadInst {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let Self { source, dest, offset, op } = self;

		let mnemonic = match op {
			LoadOpcode::Byte => "lb",
			LoadOpcode::HalfWord => "lh",
			LoadOpcode::WordLeft => "lwl",
			LoadOpcode::Word => "lw",
			LoadOpcode::ByteUnsigned => "lbu",
			LoadOpcode::HalfWordUnsigned => "lhu",
			LoadOpcode::WordRight => "lwr",
		};

		write!(f, "{mnemonic} {dest}, {offset}({source})")
	}
}
