//! Basic instructions
//!
//! This modules defines all the basic instructions from the psx.
//! They are all 1 word (`u32`) long.

// Modules
pub mod alu;
pub mod cond;
pub mod iter;
pub mod jmp;
pub mod load;
pub mod lui;
//pub mod special;
pub mod mult;
pub mod store;
pub mod sys;

// Exports
pub use alu::{AluInst, AluInstRaw};
pub use cond::{CondInst, CondRaw};
pub use iter::InstIter;
pub use jmp::{JmpInst, JmpRaw};
pub use load::{LoadInst, LoadRaw};
pub use lui::{LuiInst, LuiRaw};
//pub use special::{SpecialInst, SpecialRaw};
pub use store::{StoreInst, StoreRaw};
pub use sys::{SysInst, SysRaw};

/// All basic instructions
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[derive(derive_more::Display)]
pub enum BasicInst {
	/// Store
	Store(StoreInst),

	/// Load
	Load(LoadInst),
	
	/// Condition
	Cond(CondInst),

	/// Jump
	Jmp(JmpInst),

	/// Alu
	Alu(AluInst),

	/// Load upper immediate
	Lui(LuiInst),
	
	/// Syscalls
	Sys(SysInst),s
}

impl BasicInst {
	// TODO: MAybe extract the strings if the bitmatch macro allows for it.

	/// Decodes this instruction
	#[must_use]
	#[bitmatch::bitmatch]
	#[allow(clippy::many_single_char_names)] // `bitmatch` can only output single character names.
	pub fn decode(raw: u32) -> Option<Self> {
		Some(
			#[bitmatch]
			match raw {
				//"000000_sssss_ttttt_ddddd_iiiii_ffffff" => Self::Special(SpecialInst::decode(SpecialRaw { s, t, d, i, f })?),
				"00001p_iiiii_iiiii_iiiii_iiiii_iiiiii" => Self::Jmp(JmpInst::decode(JmpRaw { p, i })),
				"000ppp_sssss_ttttt_iiiii_iiiii_iiiiii" => Self::Cond(CondInst::decode(CondRaw { p, s, t, i })?),
				"001111_?????_ttttt_iiiii_iiiii_iiiiii" => Self::Lui(LuiInst::decode(LuiRaw { t, i })?),
				//"001ppp_sssss_ttttt_iiiii_iiiii_iiiiii" => Self::Alu(AluInst::decode(AluInstRaw { p, s, t, i })?),
				"100ppp_sssss_ttttt_iiiii_iiiii_iiiiii" => Self::Store(StoreInst::decode(StoreRaw { p, s, t, i })?),
				"101ppp_sssss_ttttt_iiiii_iiiii_iiiiii" => Self::Load(LoadInst::decode(LoadRaw { p, s, t, i })?),

				// Alu
				"000000_sssss_ttttt_ddddd_?????_ffffff" => Self::Alu(AluInst::decode(AluInstRaw::Imm(alu::AluRegInstRaw { s, t, d, f }))?),
				"001ppp_sssss_ttttt_iiiii_iiiii_iiiiii" => Self::Alu(AluInst::decode(AluInstRaw::Reg(alu::AluImmInstRaw { p, s, t, i }))?),

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
			},
		)
	}

	/// Encodes this instruction
	#[must_use]
	#[bitmatch::bitmatch]
	pub fn encode(self) -> u32 {
		#[rustfmt::skip]
		match self {
			//Self::Special(inst) => { let SpecialRaw {    s, t, d, i, f } = inst.encode(); bitpack!("000000_sssss_ttttt_ddddd_iiiii_ffffff") },
			Self::Jmp  (inst) => { let      JmpRaw { p,          i    } = inst.encode(); bitpack!("00001p_iiiii_iiiii_iiiii_iiiii_iiiiii") },
			Self::Cond (inst) => { let     CondRaw { p, s, t,    i    } = inst.encode(); bitpack!("000ppp_sssss_ttttt_iiiii_iiiii_iiiiii") },
			Self::Lui  (inst) => { let      LuiRaw {       t,    i    } = inst.encode(); bitpack!("001111_00000_ttttt_iiiii_iiiii_iiiiii") },
			//Self::Alu  (inst) => { let  AluInstRaw { p, s, t,    i    } = inst.encode(); bitpack!("001ppp_sssss_ttttt_iiiii_iiiii_iiiiii") },
			Self::Alu(_) => todo!(),
			Self::Store(inst) => { let    StoreRaw { p, s, t,    i    } = inst.encode(); bitpack!("100ppp_sssss_ttttt_iiiii_iiiii_iiiiii") },
			Self::Load (inst) => { let     LoadRaw { p, s, t,    i    } = inst.encode(); bitpack!("101ppp_sssss_ttttt_iiiii_iiiii_iiiiii") },
		}
	}
}
