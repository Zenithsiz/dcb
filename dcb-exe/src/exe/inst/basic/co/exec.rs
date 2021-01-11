//! Co-processor exec instructions

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
	/// Immediate
	pub i: u32,
}

/// Exec co-processor.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Inst {
	/// Value
	pub value: u32,
}

impl Decodable for Inst {
	type Raw = Raw;

	fn decode(raw: Self::Raw) -> Option<Self> {
		Some(Self { value: raw.i })
	}
}
impl Encodable for Inst {
	fn encode(&self) -> Self::Raw {
		Raw { i: self.value }
	}
}


impl InstFmt for Inst {
	fn mnemonic(&self) -> &'static str {
		"cop"
	}

	fn fmt(&self, _pos: crate::Pos, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let Self { dst, value } = self;

		write!(f, "lui {dst}, {value:#x}")
	}
}
