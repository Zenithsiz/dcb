//! Psx cpu instructions
//!
//! This module defines all instructions for the psx cpu, the
//! `MIPS R3051`, following [Nocash's specifications](https://problemkaputt.de/psx-spx.htm).
//!
//! The instructions are split across 3 main types,
//! - `[basic::Inst]`, which defines all 'basic' instructions, i.e. all instructions which are
//!   a single word in size and that carry no simplifications (such as `addi $a0, $a0, 10` == `addi $a0, 10`).
//! - `[pseudo::Inst]`, instructions which are decoded from basic instructions and that represent either
//!   a simplified version of an instruction, or multiple instructions (such as `la $a0, 0x80001000` == `lui $a0, 0x8000 / addiu $ao, 0x1000`).
//! - `[Directive]`, which represent data, rather than instructions, such as `dw` and `.ascii`.
//!
//! See each instruction's module for information on how they are decoded and their variants.
//!
//! Every instruction also uses the [`Register`] enum, which defines all registers in the cpu (except instruction specific
//! registers, such as the `lo` and `hi` registers).
//!
//! This module also contains the [`iter`] module, home to the [`ParseIter`] type, an iterator which parses
//! instructions from raw bytes and their position in memory.

// Modules
pub mod basic;
pub mod directive;
pub mod fmt;
pub mod iter;
pub mod pseudo;
pub mod reg;
pub mod size;

// Exports
pub use directive::Directive;
pub use fmt::InstFmt;
pub use iter::ParseIter;
pub use reg::Register;
pub use size::InstSize;

// Imports
use self::{basic::Decodable as _, pseudo::Decodable as _};
use crate::Pos;

/// An assembler instruction.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[derive(derive_more::TryInto)]
pub enum Inst<'a> {
	/// A basic instruction
	Basic(basic::Inst),

	/// A pseudo instruction
	Pseudo(pseudo::Inst),

	/// A directive
	Directive(Directive<'a>),
}

impl<'a> Inst<'a> {
	/// End of the code itself in the executable.
	pub const CODE_END: Pos = Pos(0x8006dd3c);
	/// Code range
	pub const CODE_RANGE: std::ops::Range<Pos> = Self::CODE_START..Self::CODE_END;
	/// Start of the code itself in the executable.
	pub const CODE_START: Pos = Pos(0x80013e4c);
}

impl<'a> Inst<'a> {
	/// Decodes an instruction from bytes and it's position.
	pub fn decode(pos: Pos, bytes: &'a [u8]) -> Option<Self> {
		// If we're outside of code range, or not aligned to a word, decode a directive
		if !Self::CODE_RANGE.contains(&pos) || !pos.is_word_aligned() {
			let directive = Directive::decode(pos, bytes)?;
			return Some(Self::Directive(directive));
		}

		// Else make the instruction iterator
		// Note: We fuse it to make sure that pseudo instructions don't try to skip
		//       invalid instructions.
		let mut insts = bytes
			.array_chunks::<4>()
			.copied()
			.map(u32::from_ne_bytes)
			.map_while(|word| basic::Raw::from_u32(word).and_then(basic::Inst::decode))
			.fuse();

		// Try to decode a pseudo-instruction
		if let Some(inst) = pseudo::Inst::decode(insts.clone()) {
			return Some(Self::Pseudo(inst));
		}

		// Else try to decode it as an basic instruction
		if let Some(inst) = insts.next() {
			return Some(Self::Basic(inst));
		}

		// Else read it as a directive
		Directive::decode(pos, bytes).map(Self::Directive)
	}
}

impl<'a> InstSize for Inst<'a> {
	fn size(&self) -> usize {
		match self {
			Inst::Basic(inst) => inst.size(),
			Inst::Pseudo(inst) => inst.size(),
			Inst::Directive(directive) => directive.size(),
		}
	}
}

impl<'a> InstFmt for Inst<'a> {
	fn mnemonic(&self) -> &'static str {
		match self {
			Self::Basic(inst) => inst.mnemonic(),
			Self::Pseudo(inst) => inst.mnemonic(),
			Self::Directive(directive) => directive.mnemonic(),
		}
	}

	fn fmt(&self, pos: Pos, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Self::Basic(inst) => inst.fmt(pos, f),
			Self::Pseudo(inst) => inst.fmt(pos, f),
			Self::Directive(directive) => directive.fmt(pos, f),
		}
	}
}
