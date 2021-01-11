//! Pseudo instructions
//!
//! All instructions in this module are variable length, and are decoded
//! from a starting basic instruction and remaining instruction bytes,
//! via the [`Decodable`] trait.

// Modules
pub mod alu_assign;
pub mod load_imm;
//pub mod jmp;
//pub mod load;
//pub mod move_reg;
pub mod nop;
//pub mod store;

// Imports
use super::{basic, InstFmt, InstSize};

/// A pseudo instruction
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[derive(derive_more::TryInto)]
pub enum Inst {
	/// Alu self-assign
	AluAssign(alu_assign::Inst),

	/// Load immediate
	LoadImm(load_imm::Inst),

	/// No-op
	Nop(nop::Inst),
	/*
	/// Load
	Load(LoadPseudoInst),

	/// Store
	Store(StorePseudoInst),

	/// Move register
	MoveRegPseudo(MoveRegPseudoInst),

	/// Load immediate
	LoadImm(LoadImmInst),

	*/
	/*
	/// Subtract immediate
	/// Alias for `addi $rt, $rs, -imm`
	#[display(fmt = "subi {rt}, {rs}, {imm:#x}")]
	Subi { rt: Register, rs: Register, imm: u32 },

	/// Subtract immediate sign-extended
	/// Alias for `addiu $rt, $rs, -imm`
	#[display(fmt = "subiu {rt}, {rs}, {imm:#x}")]
	Subiu { rt: Register, rs: Register, imm: u32 },

	/// Subtract immediate assign
	/// Alias for `subi $rx, $rx, imm`
	#[display(fmt = "subi {rx}, {imm:#x}")]
	SubiAssign { rx: Register, imm: u32 },

	/// Subtract immediate sign-extended assign
	/// Alias for `subiu $rx, $rx, imm`
	#[display(fmt = "subiu {rx}, {imm:#x}")]
	SubiuAssign { rx: Register, imm: u32 },
	*/
}

impl Decodable for Inst {
	fn decode(insts: impl Iterator<Item = basic::Inst> + Clone) -> Option<Self> {
		// Note: Order is important
		load_imm::Inst::decode(insts.clone())
			.map(Self::LoadImm)
			.or_else(|| alu_assign::Inst::decode(insts.clone()).map(Self::AluAssign))
			.or_else(move || nop::Inst::decode(insts).map(Self::Nop))
	}
}

impl InstSize for Inst {
	fn size(&self) -> usize {
		match self {
			Self::AluAssign(inst) => inst.size(),
			Self::LoadImm(inst) => inst.size(),
			Self::Nop(inst) => inst.size(),
		}
	}
}

impl InstFmt for Inst {
	fn mnemonic(&self) -> &'static str {
		match self {
			Self::AluAssign(inst) => inst.mnemonic(),
			Self::LoadImm(inst) => inst.mnemonic(),
			Self::Nop(inst) => inst.mnemonic(),
		}
	}

	fn fmt(&self, pos: crate::Pos, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Self::AluAssign(inst) => inst.fmt(pos, f),
			Self::LoadImm(inst) => inst.fmt(pos, f),
			Self::Nop(inst) => inst.fmt(pos, f),
		}
	}
}

/*
impl PseudoInst {
	pub fn decode(iter: InstIter<'_, impl Iterator<Item = u32> + Clone>) -> Option<Self> {
		LoadPseudoInst::decode(iter)
			.or_else(|| StorePseudoInst::decode(iter))
			.or_else(|| MoveRegPseudoInst::decode(iter))
			.or_else(|| LoadImmInst::decode(iter))
			.or_else(|| NopInst::decode(iter))
			.or_else(|| AluAssignInst::decode(iter))
			.or_else(|| JmpPseudoInst::decode(iter))
	}
}
*/

/// A decodable pseudo instruction
pub trait Decodable: InstSize + Sized {
	/// Decodes this instruction
	#[must_use]
	fn decode(insts: impl Iterator<Item = basic::Inst> + Clone) -> Option<Self>;
}

/*
/// An encodable pseudo instruction
pub trait Encodable: Decodable {
	/// Encodes this instruction
	#[must_use]
	fn encode(&self) -> Self::Raw;
}
*/
