//! Basic instructions
//!
//! All instructions in this module are a single word long, and
//! may be decoded from a `u32` via the [`Inst::decode`](<Inst as Decodable>::decode) method,
//! using the [`Decodable`] trait.

// Modules
pub mod alu;
pub mod co;
pub mod cond;
pub mod jmp;
pub mod load;
pub mod lui;
pub mod mult;
pub mod shift;
pub mod store;
pub mod sys;

// Imports
use super::InstSize;
use crate::exe::inst::InstFmt;

/// Raw instruction
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Raw {
	/// Alu
	Alu(alu::Raw),

	/// Condition
	Cond(cond::Raw),

	/// Jump
	Jmp(jmp::Raw),

	/// Load
	Load(load::Raw),

	/// Load upper immediate
	Lui(lui::Raw),

	/// Multiplication
	Mult(mult::Raw),

	/// Shift
	Shift(shift::Raw),

	/// Store
	Store(store::Raw),

	/// Syscall
	Sys(sys::Raw),

	/// Co-processor
	Co(u32),
}

impl Raw {
	/// Constructs a raw instruction from a `u32`.
	#[must_use]
	#[bitmatch::bitmatch]
	#[allow(clippy::many_single_char_names)] // `bitmatch` can only output single character names.
	pub fn from_u32(raw: u32) -> Option<Self> {
		#[rustfmt::skip]
		let raw = #[bitmatch] match raw {
			"000000_?????_ttttt_ddddd_iiiii_0000ff" => Self::Shift(shift::Raw::Imm(shift::imm::Raw {       t, d, i, f })),
			"000000_sssss_ttttt_ddddd_?????_0001ff" => Self::Shift(shift::Raw::Reg(shift::reg::Raw {    s, t, d,    f })),
			"000000_sssss_?????_ddddd_?????_00100f" => Self::Jmp  (jmp  ::Raw::Reg(jmp  ::reg::Raw {    s,    d,    f })),
			"000000_ccccc_ccccc_ccccc_ccccc_00110f" => Self::Sys  (                sys  ::     Raw {             c, f } ),
			"000000_sssss_ttttt_ddddd_?????_01ffff" => Self::Mult (                mult ::     Raw {    s, t, d,    f } ),
			"000000_sssss_ttttt_ddddd_?????_10ffff" => Self::Alu  (alu  ::Raw::Reg(alu  ::reg::Raw {    s, t, d,    f })),
			"00001p_iiiii_iiiii_iiiii_iiiii_iiiiii" => Self::Jmp  (jmp  ::Raw::Imm(jmp  ::imm::Raw { p,          i    })),
			"000ppp_sssss_ttttt_iiiii_iiiii_iiiiii" => Self::Cond (                cond ::     Raw { p, s, t,    i    } ),
			"001111_?????_ttttt_iiiii_iiiii_iiiiii" => Self::Lui  (                lui  ::     Raw {       t,    i    } ),
			"001ppp_sssss_ttttt_iiiii_iiiii_iiiiii" => Self::Alu  (alu  ::Raw::Imm(alu  ::imm::Raw { p, s, t,    i    })),
			"100ppp_sssss_ttttt_iiiii_iiiii_iiiiii" => Self::Load (                load ::     Raw { p, s, t,    i    } ),
			"101ppp_sssss_ttttt_iiiii_iiiii_iiiiii" => Self::Store(                store::     Raw { p, s, t,    i    } ),
			"?1????_?????_?????_?????_?????_??????" => Self::Co   (raw),
			_ => return None,
		};

		Some(raw)
	}

	/// Encodes this raw as a `u32`
	#[must_use]
	#[bitmatch::bitmatch]
	#[rustfmt::skip]
	pub const fn as_u32(&self) -> u32 {
		match *self {
			Self::Shift(shift::Raw::Imm(shift::imm::Raw {       t, d, i, f })) => bitpack!("000000_?????_ttttt_ddddd_iiiii_0000ff"),
			Self::Shift(shift::Raw::Reg(shift::reg::Raw {    s, t, d,    f })) => bitpack!("000000_sssss_ttttt_ddddd_?????_0001ff"),
			Self::Jmp  (jmp  ::Raw::Reg(jmp  ::reg::Raw {    s,    d,    f })) => bitpack!("000000_sssss_?????_ddddd_?????_00100f"),
			Self::Sys  (                sys  ::     Raw {             c, f } ) => bitpack!("000000_ccccc_ccccc_ccccc_ccccc_00110f"),
			Self::Mult (                mult ::     Raw {    s, t, d,    f } ) => bitpack!("000000_sssss_ttttt_ddddd_?????_01ffff"),
			Self::Alu  (alu  ::Raw::Reg(alu  ::reg::Raw {    s, t, d,    f })) => bitpack!("000000_sssss_ttttt_ddddd_?????_10ffff"),
			Self::Jmp  (jmp  ::Raw::Imm(jmp  ::imm::Raw { p,          i    })) => bitpack!("00001p_iiiii_iiiii_iiiii_iiiii_iiiiii"),
			Self::Cond (                cond ::     Raw { p, s, t,    i    } ) => bitpack!("000ppp_sssss_ttttt_iiiii_iiiii_iiiiii"),
			Self::Lui  (                lui  ::     Raw {       t,    i    } ) => bitpack!("001111_?????_ttttt_iiiii_iiiii_iiiiii"),
			Self::Alu  (alu  ::Raw::Imm(alu  ::imm::Raw { p, s, t,    i    })) => bitpack!("001ppp_sssss_ttttt_iiiii_iiiii_iiiiii"),
			Self::Load (                load ::     Raw { p, s, t,    i    } ) => bitpack!("100ppp_sssss_ttttt_iiiii_iiiii_iiiiii"),
			Self::Store(                store::     Raw { p, s, t,    i    } ) => bitpack!("101ppp_sssss_ttttt_iiiii_iiiii_iiiiii"),
			Self::Co   (raw) => raw,
		}
	}
}


/// All basic instructions
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[derive(derive_more::TryInto)]
pub enum Inst {
	/// Alu
	Alu(alu::Inst),

	/// Condition
	Cond(cond::Inst),

	/// Jump
	Jmp(jmp::Inst),

	/// Load
	Load(load::Inst),

	/// Load upper immediate
	Lui(lui::Inst),

	/// Multiplication
	Mult(mult::Inst),

	/// Shift
	Shift(shift::Inst),

	/// Store
	Store(store::Inst),

	/// Syscall
	Sys(sys::Inst),

	/// Co-processor
	Co(co::Inst),
}


impl Decodable for Inst {
	type Raw = Raw;

	fn decode(raw: Self::Raw) -> Option<Self> {
		#[rustfmt::skip]
		let inst =
		match raw {
			Raw::Alu  (raw) => Self::Alu  (alu  ::Inst::decode(raw)?),
			Raw::Cond (raw) => Self::Cond (cond ::Inst::decode(raw)?),
			Raw::Jmp  (raw) => Self::Jmp  (jmp  ::Inst::decode(raw)?),
			Raw::Load (raw) => Self::Load (load ::Inst::decode(raw)?),
			Raw::Lui  (raw) => Self::Lui  (lui  ::Inst::decode(raw)?),
			Raw::Mult (raw) => Self::Mult (mult ::Inst::decode(raw)?),
			Raw::Shift(raw) => Self::Shift(shift::Inst::decode(raw)?),
			Raw::Store(raw) => Self::Store(store::Inst::decode(raw)?),
			Raw::Sys  (raw) => Self::Sys  (sys  ::Inst::decode(raw)?),
			Raw::Co   (raw) => Self::Co   (co   ::Inst::decode(raw)?),
		};

		Some(inst)
	}
}

impl Encodable for Inst {
	#[rustfmt::skip]
	fn encode(&self) -> Self::Raw {
		match self {
			Self::Alu  (inst) => Raw::Alu  (inst.encode()),
			Self::Cond (inst) => Raw::Cond (inst.encode()),
			Self::Jmp  (inst) => Raw::Jmp  (inst.encode()),
			Self::Load (inst) => Raw::Load (inst.encode()),
			Self::Lui  (inst) => Raw::Lui  (inst.encode()),
			Self::Mult (inst) => Raw::Mult (inst.encode()),
			Self::Shift(inst) => Raw::Shift(inst.encode()),
			Self::Store(inst) => Raw::Store(inst.encode()),
			Self::Sys  (inst) => Raw::Sys  (inst.encode()),
			Self::Co   (inst) => Raw::Co   (inst.encode()),
		}
	}
}

// Any basic decodable instruction is 4 bytes
impl<T: Decodable> InstSize for T {
	fn size(&self) -> usize {
		4
	}
}

impl InstFmt for Inst {
	#[rustfmt::skip]
	fn fmt(&self, pos: crate::Pos, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Self::Alu  (inst) => inst.fmt(pos, f),
			Self::Cond (inst) => inst.fmt(pos, f),
			Self::Jmp  (inst) => inst.fmt(pos, f),
			Self::Load (inst) => inst.fmt(pos, f),
			Self::Lui  (inst) => inst.fmt(pos, f),
			Self::Mult (inst) => inst.fmt(pos, f),
			Self::Shift(inst) => inst.fmt(pos, f),
			Self::Store(inst) => inst.fmt(pos, f),
			Self::Sys  (inst) => inst.fmt(pos, f),
			Self::Co   (inst) => inst.fmt(pos, f),
		}
	}
}

/// A decodable basic instruction
pub trait Decodable: Sized {
	/// 'Raw' type to parse from
	type Raw;

	/// Decodes this instruction
	#[must_use]
	fn decode(raw: Self::Raw) -> Option<Self>;
}

/// An encodable basic instruction
pub trait Encodable: Decodable {
	/// Encodes this instruction
	#[must_use]
	fn encode(&self) -> Self::Raw;
}
