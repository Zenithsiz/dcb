//! Pseudo instructions
//!
//! This modules defines all the pseudo instructions usually
//! used in mips. They are variable length.

// Modules
pub mod alu_assign;
pub mod jmp;
pub mod load;
pub mod load_imm;
pub mod move_reg;
pub mod nop;
pub mod store;

// Exports
pub use alu_assign::AluAssignInst;
pub use jmp::JmpPseudoInst;
pub use load::LoadPseudoInst;
pub use load_imm::LoadImmInst;
pub use move_reg::MoveRegPseudoInst;
pub use nop::NopInst;
pub use store::StorePseudoInst;

// Imports
use crate::exe::instruction::basic::InstIter;

/// A pseudo instruction
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[derive(derive_more::Display)]
pub enum PseudoInst {
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

	/// Alu self-assign
	AluAssign(AluAssignInst),
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
