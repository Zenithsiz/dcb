//! Lui instruction

// Imports
use super::ModifiesReg;
use crate::inst::{
	basic::{Decodable, Encodable},
	InstFmt, Register,
};
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
pub struct Inst {
	/// Destination register, `rt`
	pub dst: Register,

	/// Value
	pub value: u16,
}

impl Decodable for Inst {
	type Raw = Raw;

	fn decode(raw: Self::Raw) -> Option<Self> {
		Some(Self {
			dst:   Register::new(raw.t)?,
			value: raw.i.truncated::<u16>(),
		})
	}
}
impl Encodable for Inst {
	fn encode(&self) -> Self::Raw {
		Raw {
			t: self.dst.idx(),
			i: self.value.zero_extended::<u32>(),
		}
	}
}


impl InstFmt for Inst {
	fn fmt(&self, _pos: crate::Pos, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let Self { dst, value } = self;

		write!(f, "lui {dst}, {value:#x}")
	}
}

impl ModifiesReg for Inst {
	fn modifies_reg(&self, reg: Register) -> bool {
		self.dst == reg
	}
}
