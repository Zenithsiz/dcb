//! Lui instruction

// Imports
use crate::exe::inst::Register;
use int_conv::{Truncated, ZeroExtended};

/// Raw representation
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Raw {
	/// Rt
	pub t: u32,

	/// Immediate
	pub i: u32,
}

/// Load instructions
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[derive(derive_more::Display)]
#[display(fmt = "lui {dst}, {value:#x}")]
pub struct Inst {
	/// Destination register, `rt`
	pub dst: Register,

	/// Value
	pub value: u16,
}

impl Inst {
	/// Decodes this instruction
	#[must_use]
	pub fn decode(raw: Raw) -> Option<Self> {
		Some(Self {
			dst:   Register::new(raw.t)?,
			value: raw.i.truncated::<u16>(),
		})
	}

	/// Encodes this instruction
	#[must_use]
	pub fn encode(self) -> Raw {
		Raw {
			t: self.dst.idx(),
			i: self.value.zero_extended::<u32>(),
		}
	}
}
