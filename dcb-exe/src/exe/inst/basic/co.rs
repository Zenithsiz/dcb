//! Co-processor instructions

// Modules
//pub mod exec;
//pub mod move_reg;
//pub mod cond;
//pub mod load;
//pub mod store;

// Imports
use crate::exe::inst::{
	basic::{Decodable, Encodable},
	InstFmt, Register,
};
use int_conv::{Signed, Truncated};

/// Co-processor instruction kind
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Kind {
	/// Execute
	CopN {
		/// Command to execute
		imm: u32,
	},

	/// Move data register from co-processor
	MfcN {
		/// Destination
		dst: Register,

		/// Source
		src: Register,
	},

	/// Move control register from co-processor
	CfcN {
		/// Destination
		dst: Register,

		/// Source
		src: Register,
	},
	/// Move data register to co-processor
	MtcN {
		/// Destination
		dst: Register,

		/// Source
		src: Register,
	},
	/// Move control register to co-processor
	CtcN {
		/// Destination
		dst: Register,

		/// Source
		src: Register,
	},
	/// Branch if true
	BcNf {
		/// Offset
		offset: i16,
	},

	/// Branch if false
	BcNt {
		/// Offset
		offset: i16,
	},

	/// Load co-processor
	LwcN {
		/// Destination
		dst: Register,

		/// Source
		src: Register,

		/// offset
		offset: i16,
	},

	/// Store co-processor
	SwcN {
		/// Destination
		dst: Register,

		/// Source
		src: Register,

		/// offset
		offset: i16,
	},
}

/// Store instructions
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Inst {
	/// Co-processor number
	pub n: u32,

	/// Kind
	pub kind: Kind,
}

impl Decodable for Inst {
	type Raw = u32;

	#[bitmatch::bitmatch]
	fn decode(raw: Self::Raw) -> Option<Self> {
		#[rustfmt::skip]
		let (n, kind) = #[bitmatch] match raw {
			"0100nn_1iiii_iiiii_iiiii_iiiii_iiiiii" => (n, Kind::CopN { imm: i }),
			"0100nn_00000_ttttt_ddddd_?????_000000" => (n, Kind::MfcN { dst: Register::new(t)?, src: Register::new(d)? }),
			"0100nn_00010_ttttt_ddddd_?????_000000" => (n, Kind::CfcN { dst: Register::new(t)?, src: Register::new(d)? }),
			"0100nn_00100_ttttt_ddddd_?????_000000" => (n, Kind::MtcN { dst: Register::new(d)?, src: Register::new(t)? }),
			"0100nn_00110_ttttt_ddddd_?????_000000" => (n, Kind::CtcN { dst: Register::new(d)?, src: Register::new(t)? }),
			"0100nn_01000_00000_iiiii_iiiii_iiiiii" => (n, Kind::BcNf { offset: i.truncated::<u16>().as_signed() }),
			"0100nn_01000_00001_iiiii_iiiii_iiiiii" => (n, Kind::BcNt { offset: i.truncated::<u16>().as_signed() }),
			"1100nn_sssss_ttttt_iiiii_iiiii_iiiiii" => (n, Kind::LwcN { dst: Register::new(t)?, src: Register::new(s)?, offset: i.truncated::<u16>().as_signed() }),
			"1110nn_sssss_ttttt_iiiii_iiiii_iiiiii" => (n, Kind::SwcN { dst: Register::new(s)?, src: Register::new(t)?, offset: i.truncated::<u16>().as_signed() }),
			_ => return None,
		};

		Some(Self { n, kind })
	}
}
impl Encodable for Inst {
	fn encode(&self) -> Self::Raw {
		todo!();
	}
}

impl InstFmt for Inst {
	fn fmt(&self, _pos: crate::Pos, _f: &mut std::fmt::Formatter) -> std::fmt::Result {
		todo!();
	}
}
