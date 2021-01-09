//! Lui instruction

// Imports
use crate::exe::inst::{
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
	fn mnemonic(&self) -> &'static str {
		"lui"
	}

	fn fmt(&self, _pos: crate::Pos, _bytes: &[u8], f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let Self { dst, value } = self;

		write!(f, "lui {dst}, {value:#x}")
	}
}
