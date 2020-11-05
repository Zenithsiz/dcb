//! Lui instruction

// Imports
use crate::game::exe::instruction::Register;
use int_conv::{Truncated, ZeroExtended};

/// Raw representation
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct LuiRaw {
	/// Rt
	pub t: u32,

	/// Immediate
	pub i: u32,
}

/// Load instructions
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[derive(derive_more::Display)]
#[display(fmt = "lui {dst}, {value:#x}")]
pub struct LuiInst {
	/// Destination register, `rt`
	pub dst: Register,

	/// Value
	pub value: u16,
}

impl LuiInst {
	/// Decodes this instruction
	#[must_use]
	pub fn decode(raw: LuiRaw) -> Option<Self> {
		Some(Self {
			dst:   Register::new(raw.t)?,
			value: raw.i.truncated::<u16>(),
		})
	}

	/// Encodes this instruction
	#[must_use]
	pub fn encode(self) -> LuiRaw {
		LuiRaw {
			t: self.dst.idx(),
			i: self.value.zero_extended::<u32>(),
		}
	}
}
