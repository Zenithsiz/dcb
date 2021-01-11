//! Pseudo instructions
//!
//! All instructions in this module are variable length, and are decoded
//! from a starting basic instruction and remaining instruction bytes,
//! via the [`Decodable`] trait.

// Modules
pub mod alu_assign;
pub mod jmp;
pub mod load;
pub mod load_imm;
pub mod move_reg;
pub mod nop;
pub mod shift_assign;
pub mod store;

// Imports
use super::{basic, InstFmt, InstSize};

/// A pseudo instruction
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[derive(derive_more::TryInto)]
pub enum Inst {
	/// Alu self-assign
	AluAssign(alu_assign::Inst),

	/// Shift self-assign
	ShiftAssign(shift_assign::Inst),

	/// Load immediate
	LoadImm(load_imm::Inst),

	/// No-op
	Nop(nop::Inst),

	/// Move register
	MoveReg(move_reg::Inst),

	/// Jump
	Jmp(jmp::Inst),

	/// Load
	Load(load::Inst),

	/// Store
	Store(store::Inst),
}

impl Decodable for Inst {
	#[rustfmt::skip]
	fn decode(insts: impl Iterator<Item = basic::Inst> + Clone) -> Option<Self> {
		                 load_imm    ::Inst::decode(insts.clone()).map(Self::LoadImm    )
		.or_else(     || nop         ::Inst::decode(insts.clone()).map(Self::Nop        )) // Note: Nop must come before `shift_assign`
		.or_else(     || alu_assign  ::Inst::decode(insts.clone()).map(Self::AluAssign  ))
		.or_else(     || shift_assign::Inst::decode(insts.clone()).map(Self::ShiftAssign))
		.or_else(     || jmp         ::Inst::decode(insts.clone()).map(Self::Jmp        ))
		.or_else(     || load        ::Inst::decode(insts.clone()).map(Self::Load       ))
		.or_else(     || store       ::Inst::decode(insts.clone()).map(Self::Store      ))
		.or_else(move || move_reg    ::Inst::decode(       insts        ).map(Self::MoveReg    ))
	}
}

impl InstSize for Inst {
	fn size(&self) -> usize {
		match self {
			Self::AluAssign(inst) => inst.size(),
			Self::ShiftAssign(inst) => inst.size(),
			Self::LoadImm(inst) => inst.size(),
			Self::Nop(inst) => inst.size(),
			Self::MoveReg(inst) => inst.size(),
			Self::Jmp(inst) => inst.size(),
			Self::Load(inst) => inst.size(),
			Self::Store(inst) => inst.size(),
		}
	}
}

impl InstFmt for Inst {
	fn mnemonic(&self) -> &'static str {
		match self {
			Self::AluAssign(inst) => inst.mnemonic(),
			Self::ShiftAssign(inst) => inst.mnemonic(),
			Self::LoadImm(inst) => inst.mnemonic(),
			Self::Nop(inst) => inst.mnemonic(),
			Self::MoveReg(inst) => inst.mnemonic(),
			Self::Jmp(inst) => inst.mnemonic(),
			Self::Load(inst) => inst.mnemonic(),
			Self::Store(inst) => inst.mnemonic(),
		}
	}

	fn fmt(&self, pos: crate::Pos, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Self::AluAssign(inst) => inst.fmt(pos, f),
			Self::ShiftAssign(inst) => inst.fmt(pos, f),
			Self::LoadImm(inst) => inst.fmt(pos, f),
			Self::Nop(inst) => inst.fmt(pos, f),
			Self::MoveReg(inst) => inst.fmt(pos, f),
			Self::Jmp(inst) => inst.fmt(pos, f),
			Self::Load(inst) => inst.fmt(pos, f),
			Self::Store(inst) => inst.fmt(pos, f),
		}
	}
}

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
