//! Pseudo instructions
//!
//! All instructions in this module are variable length, and are decoded
//! from a starting basic instruction and remaining instruction bytes,
//! via the [`Decodable`] trait.

// Modules
pub mod alu_assign;
//pub mod jmp;
//pub mod load;
//pub mod load_imm;
//pub mod move_reg;
//pub mod nop;
//pub mod store;

// Imports
use super::{basic, InstFmt};

/// A pseudo instruction
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Inst {
	/// Alu self-assign
	AluAssign(alu_assign::Inst),
	/*
	/// Load
	Load(LoadPseudoInst),

	/// Store
	Store(StorePseudoInst),

	/// Move register
	MoveRegPseudo(MoveRegPseudoInst),

	/// Load immediate
	LoadImm(LoadImmInst),

	/// No-op
	Nop(NopInst),

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
		alu_assign::Inst::decode(insts).map(Self::AluAssign)
	}

	fn size(&self) -> u32 {
		match self {
			Self::AluAssign(inst) => inst.size(),
		}
	}
}

impl InstFmt for Inst {
	fn mnemonic(&self) -> &'static str {
		match self {
			Self::AluAssign(inst) => inst.mnemonic(),
		}
	}

	fn fmt(&self, pos: crate::Pos, bytes: &[u8], f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Self::AluAssign(inst) => inst.fmt(pos, bytes, f),
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
pub trait Decodable: Sized {
	/// Decodes this instruction
	#[must_use]
	fn decode(insts: impl Iterator<Item = basic::Inst> + Clone) -> Option<Self>;

	/// Returns how many _words_ long this instruction is
	fn size(&self) -> u32;
}

/*
/// An encodable pseudo instruction
pub trait Encodable: Decodable {
	/// Encodes this instruction
	#[must_use]
	fn encode(&self) -> Self::Raw;
}
*/
