//! Lui instruction

// Imports
use super::ModifiesReg;
use crate::inst::{
	basic::{Decodable, Encodable},
	InstFmt, Register,
};
use int_conv::{Truncated, ZeroExtended};

/// Load instructions
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Inst {
	/// Destination register, `rt`
	pub dst: Register,

	/// Value
	pub value: u16,
}

impl Decodable for Inst {
	type Raw = u32;

	#[bitmatch::bitmatch]
	fn decode(raw: Self::Raw) -> Option<Self> {
		let [t, i] = #[bitmatch]
		match raw {
			"001111_?????_ttttt_iiiii_iiiii_iiiiii" => [t, i],
			_ => return None,
		};

		Some(Self {
			dst:   Register::new(t)?,
			value: i.truncated::<u16>(),
		})
	}
}
impl Encodable for Inst {
	#[bitmatch::bitmatch]
	fn encode(&self) -> Self::Raw {
		let t = self.dst.idx();
		let i = self.value.zero_extended::<u32>();

		bitpack!("001111_?????_ttttt_iiiii_iiiii_iiiiii")
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
