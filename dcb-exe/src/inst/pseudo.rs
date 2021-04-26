//! Pseudo instructions
//!
//! All instructions in this module are variable length, and are decoded
//! from a starting basic instruction and remaining instruction bytes,
//! via the [`Decodable`] trait.

// Modules
pub mod load;
pub mod load_imm;
pub mod move_reg;
pub mod nop;
pub mod store;

// Imports
use super::{basic, DisplayCtx, InstDisplay, InstFmt, InstFmtArg, InstSize};
use core::fmt;

/// A pseudo instruction
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[derive(derive_more::TryInto)]
pub enum Inst {
	/// Load immediate
	LoadImm(load_imm::Inst),

	/// No-op
	Nop(nop::Inst),

	/// Move register
	MoveReg(move_reg::Inst),

	/// Load
	Load(load::Inst),

	/// Store
	Store(store::Inst),
}

impl Decodable for Inst {
	#[rustfmt::skip]
	fn decode(insts: impl Iterator<Item = basic::Inst> + Clone) -> Option<Self> {
		                 load_imm    ::Inst::decode(insts.clone()).map(Self::LoadImm    )
		.or_else(     || nop         ::Inst::decode(insts.clone()).map(Self::Nop        ))
		.or_else(     || load        ::Inst::decode(insts.clone()).map(Self::Load       ))
		.or_else(     || store       ::Inst::decode(insts.clone()).map(Self::Store      ))
		.or_else(move || move_reg    ::Inst::decode(       insts        ).map(Self::MoveReg    ))
	}
}

impl Encodable for Inst {
	type Iterator = impl IntoIterator<Item = basic::Inst>;

	#[auto_enums::auto_enum(Iterator)]
	fn encode(&self) -> Self::Iterator {
		match self {
			Inst::LoadImm(inst) => inst.encode(),
			Inst::Nop(inst) => inst.encode(),
			Inst::MoveReg(inst) => inst.encode(),
			Inst::Load(inst) => inst.encode(),
			Inst::Store(inst) => inst.encode(),
		}
	}
}

impl<'a> InstDisplay<'a> for Inst {
	type Args = impl Iterator<Item = InstFmtArg<'a>>;
	type Mnemonic = impl fmt::Display;

	#[auto_enums::auto_enum(Display)]
	#[rustfmt::skip]
	fn mnemonic<Ctx: DisplayCtx>(&'a self, ctx: &Ctx) -> Self::Mnemonic {
		match self {
			Inst::LoadImm(inst) => inst.mnemonic(ctx),
			Inst::Nop    (inst) => inst.mnemonic(ctx),
			Inst::MoveReg(inst) => inst.mnemonic(ctx),
			Inst::Load   (inst) => inst.mnemonic(ctx),
			Inst::Store  (inst) => inst.mnemonic(ctx),
		}
	}

	#[auto_enums::auto_enum(Iterator)]
	#[rustfmt::skip]
	fn args<Ctx: DisplayCtx>(&'a self, ctx: &Ctx) -> Self::Args {
		match self {
			Inst::LoadImm(inst) => inst.args(ctx),
			Inst::Nop    (inst) => inst.args(ctx),
			Inst::MoveReg(inst) => inst.args(ctx),
			Inst::Load   (inst) => inst.args(ctx),
			Inst::Store  (inst) => inst.args(ctx),
		}
	}
}

impl InstSize for Inst {
	fn size(&self) -> usize {
		match self {
			Self::LoadImm(inst) => inst.size(),
			Self::Nop(inst) => inst.size(),
			Self::MoveReg(inst) => inst.size(),
			Self::Load(inst) => inst.size(),
			Self::Store(inst) => inst.size(),
		}
	}
}

impl InstFmt for Inst {
	fn fmt(&self, pos: crate::Pos, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Self::LoadImm(inst) => inst.fmt(pos, f),
			Self::Nop(inst) => inst.fmt(pos, f),
			Self::MoveReg(inst) => inst.fmt(pos, f),
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

/// An encodable pseudo instruction
pub trait Encodable {
	/// Iterator type
	type Iterator: IntoIterator<Item = basic::Inst>;

	/// Encodes this instruction as basic instructions
	#[must_use]
	fn encode(&self) -> Self::Iterator;
}
