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
pub mod target;

// Exports
pub use directive::Directive;
pub use fmt::{InstFmt, InstTargetFmt};
pub use iter::ParseIter;
pub use reg::Register;
pub use size::InstSize;
pub use target::InstTarget;

// Imports
use self::{basic::Decodable as _, directive::DecodeWithDataError, pseudo::Decodable as _};
use super::{Data, DataTable, FuncTable};
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

/// Error type for [`Inst::decode`]
#[derive(Debug, thiserror::Error)]
pub enum DecodeError<'a> {
	/// Invalid data location to read from
	#[error("Attempted to decode instruction from within data location")]
	InvalidDataLocation {
		/// The data location
		data: &'a Data,

		/// Underlying error
		err: DecodeWithDataError,
	},

	/// Bytes is empty
	#[error("No more bytes")]
	NoBytes,
}

impl<'a> Inst<'a> {
	/// Decodes an instruction from bytes and it's position.
	pub fn decode(pos: Pos, bytes: &'a [u8], data_table: &'a DataTable, _func_table: &'a FuncTable) -> Result<Self, DecodeError<'a>> {
		// If `bytes` is empty, return Err
		if bytes.is_empty() {
			return Err(DecodeError::NoBytes);
		}

		// If we're contained in some data, check it's type so we can read it
		if let Some(data) = data_table.get_containing(pos) {
			return Directive::decode_with_data(pos, bytes, &data.ty, data.pos)
				.map(Self::Directive)
				.map_err(|err| DecodeError::InvalidDataLocation { data, err });
		}

		// TODO: Check functions

		// If we're not aligned to a word, decode a directive
		if !pos.is_word_aligned() {
			let directive = Directive::decode(pos, bytes).ok_or(DecodeError::NoBytes)?;
			return Ok(Self::Directive(directive));
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
			return Ok(Self::Pseudo(inst));
		}

		// Else try to decode it as an basic instruction
		if let Some(inst) = insts.next() {
			return Ok(Self::Basic(inst));
		}

		// Else read it as a directive
		Directive::decode(pos, bytes).map(Self::Directive).ok_or(DecodeError::NoBytes)
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
	fn fmt(&self, pos: Pos, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Self::Basic(inst) => inst.fmt(pos, f),
			Self::Pseudo(inst) => inst.fmt(pos, f),
			Self::Directive(directive) => directive.fmt(pos, f),
		}
	}
}
