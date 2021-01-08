//! Basic instructions
//!
//! This modules defines all the basic instructions from the psx.
//! They are all 1 word (`u32`) long.

// Modules
pub mod alu;
pub mod cond;
pub mod jmp;
pub mod load;
pub mod lui;
pub mod mult;
pub mod store;
pub mod sys;

/// All basic instructions
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[derive(derive_more::Display)]
pub enum Inst {
	/// Store
	Store(store::Inst),

	/// Load
	Load(load::Inst),

	/// Condition
	Cond(cond::Inst),

	/// Jump
	Jmp(jmp::Inst),

	/// Alu
	Alu(alu::Inst),

	/// Load upper immediate
	Lui(lui::Inst),

	/// Syscall
	Sys(sys::Inst),
}

impl Decodable for Inst {
	type Raw = u32;

	#[bitmatch::bitmatch]
	#[allow(clippy::many_single_char_names)] // `bitmatch` can only output single character names.
	fn decode(raw: Self::Raw) -> Option<Self> {
		let inst = #[bitmatch]
		match raw {
			// Jump
			"00001p_iiiii_iiiii_iiiii_iiiii_iiiiii" => Self::Jmp(jmp::Inst::decode_from(jmp::imm::Raw { p, i })?),
			"000000_sssss_?????_ddddd_?????_00100f" => Self::Jmp(jmp::Inst::decode_from(jmp::reg::Raw { s, d, f })?),

			"000ppp_sssss_ttttt_iiiii_iiiii_iiiiii" => Self::Cond(cond::Inst::decode(cond::Raw { p, s, t, i })?),
			"001111_?????_ttttt_iiiii_iiiii_iiiiii" => Self::Lui(lui::Inst::decode(lui::Raw { t, i })?),

			// Alu
			"000000_sssss_ttttt_ddddd_?????_10ffff" => Self::Alu(alu::Inst::decode_from(alu::reg::Raw { s, t, d, f })?),
			"001ppp_sssss_ttttt_iiiii_iiiii_iiiiii" => Self::Alu(alu::Inst::decode_from(alu::imm::Raw { p, s, t, i })?),

			// Syscall
			"000000_ccccc_ccccc_ccccc_ccccc_00110f" => Self::Sys(sys::Inst::decode(sys::Raw { c, f })?),

			// Store / Load
			"100ppp_sssss_ttttt_iiiii_iiiii_iiiiii" => Self::Store(store::Inst::decode(store::Raw { p, s, t, i })?),
			"101ppp_sssss_ttttt_iiiii_iiiii_iiiiii" => Self::Load(load::Inst::decode(load::Raw { p, s, t, i })?),

			// TODO: Remaining instructions, such as shift

			/*
			"0100nn_1iiii_iiiii_iiiii_iiiii_iiiiii" => CopN { n: n.truncate(), imm: i},
			"0100nn_00000_ttttt_ddddd_?????_000000" => MfcN { n: n.truncate(), rt: reg(t)?, rd: reg(d)? },
			"0100nn_00010_ttttt_ddddd_?????_000000" => CfcN { n: n.truncate(), rt: reg(t)?, rd: reg(d)? },
			"0100nn_00100_ttttt_ddddd_?????_000000" => MtcN { n: n.truncate(), rt: reg(t)?, rd: reg(d)? },
			"0100nn_00110_ttttt_ddddd_?????_000000" => CtcN { n: n.truncate(), rt: reg(t)?, rd: reg(d)? },
			"0100nn_01000_00000_iiiii_iiiii_iiiiii" => BcNf { n: n.truncate(), target: i.truncate() },
			"0100nn_01000_00001_iiiii_iiiii_iiiiii" => BcNt { n: n.truncate(), target: i.truncate() },
			"1100nn_sssss_ttttt_iiiii_iiiii_iiiiii" => LwcN { n: n.truncate(), rs: reg(s)?, rt: reg(t)?, imm: i.truncate() },
			"1110nn_sssss_ttttt_iiiii_iiiii_iiiiii" => SwcN { n: n.truncate(), rs: reg(s)?, rt: reg(t)?, imm: i.truncate() },
			*/
			_ => return None,
		};

		Some(inst)
	}
}

impl Encodable for Inst {
	#[bitmatch::bitmatch]
	fn encode(&self) -> u32 {
		match self {
			Self::Jmp(inst) => match inst.encode() {
				jmp::Raw::Imm(jmp::imm::Raw { p, i }) => bitpack!("00001p_iiiii_iiiii_iiiii_iiiii_iiiiii"),
				jmp::Raw::Reg(jmp::reg::Raw { s, d, f }) => bitpack!("000000_sssss_?????_ddddd_?????_00100f"),
			},
			Self::Cond(inst) => {
				let cond::Raw { p, s, t, i } = inst.encode();
				bitpack!("000ppp_sssss_ttttt_iiiii_iiiii_iiiiii")
			},
			Self::Lui(inst) => {
				let lui::Raw { t, i } = inst.encode();
				bitpack!("001111_?????_ttttt_iiiii_iiiii_iiiiii")
			},
			Self::Alu(inst) => match inst.encode() {
				alu::Raw::Imm(alu::imm::Raw { p, s, t, i }) => bitpack!("001ppp_sssss_ttttt_iiiii_iiiii_iiiiii"),
				alu::Raw::Reg(alu::reg::Raw { s, t, d, f }) => bitpack!("000000_sssss_ttttt_ddddd_?????_10ffff"),
			},
			Self::Sys(inst) => {
				let sys::Raw { c, f } = inst.encode();
				bitpack!("000000_ccccc_ccccc_ccccc_ccccc_00110f")
			},
			Self::Store(inst) => {
				let store::Raw { p, s, t, i } = inst.encode();
				bitpack!("100ppp_sssss_ttttt_iiiii_iiiii_iiiiii")
			},
			Self::Load(inst) => {
				let load::Raw { p, s, t, i } = inst.encode();
				bitpack!("101ppp_sssss_ttttt_iiiii_iiiii_iiiiii")
			},
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

	/// Decodes this instruction from any type that can be converted into the raw form
	#[must_use]
	fn decode_from(raw: impl Into<Self::Raw>) -> Option<Self> {
		Self::decode(raw.into())
	}
}

/// An encodable basic instruction
pub trait Encodable: Decodable {
	/// Encodes this instruction
	#[must_use]
	fn encode(&self) -> Self::Raw;
}