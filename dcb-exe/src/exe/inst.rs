//! Psx cpu instructions

// Modules
pub mod basic;
pub mod directive;
pub mod pseudo;
pub mod raw;
pub mod reg;

// Exports
pub use directive::Directive;
pub use raw::Raw;
pub use reg::Register;

// Imports
use crate::Pos;
use dcb_util::NextFromBytes;

/// An assembler instruction
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Inst {
	/// A basic instruction
	Basic(basic::Inst),

	/// A pseudo instruction
	Pseudo(pseudo::PseudoInst),

	/// A directive
	Directive(Directive),
}

impl Inst {
	/// End of the code itself in the executable.
	pub const CODE_END: Pos = Pos(0x8006dd3c);
	/// Code range
	pub const CODE_RANGE: std::ops::Range<Pos> = Self::CODE_START..Self::CODE_END;
	/// Start of the code itself in the executable.
	pub const CODE_START: Pos = Pos(0x80013e4c);
}

/// Parsing iterator, reads instructions from a `[u8]` slice
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct ParseIter<'a> {
	/// Remaining bytes
	bytes: &'a [u8],

	/// Starting position of bytes
	cur_pos: Pos,
}

impl<'a> ParseIter<'a> {
	/// Creates a new parsing iterator
	#[must_use]
	pub const fn new(bytes: &'a [u8], start_pos: Pos) -> Self {
		Self { bytes, cur_pos: start_pos }
	}
}

impl<'a> Iterator for ParseIter<'a> {
	type Item = (Pos, Inst);

	#[allow(clippy::as_conversions, clippy::cast_possible_truncation)] // Byte lengths will always fit into a `u32`, as `self.bytes.len()` is always smaller than `u32`.
	#[allow(clippy::indexing_slicing)] // Our lengths will always be smaller than the bytes array they are used to index.
	fn next(&mut self) -> Option<Self::Item> {
		// If we're outside of code range, decode a directive
		if !Inst::CODE_RANGE.contains(&self.cur_pos) {
			let (directive, len) = Directive::decode(self.cur_pos, self.bytes)?;
			self.bytes = &self.bytes[len..];
			let pos = self.cur_pos;
			self.cur_pos += len as u32;
			return Some((pos, Inst::Directive(directive)));
		}

		// Else decode an instruction, falling back to a directive if unable to
		match self.bytes.next_u32().and_then(basic::Inst::decode) {
			// If we got one, update our bytes and check if it's a pseudo instruction
			Some(inst) => {
				self.bytes = &self.bytes[4..];
				let pos = self.cur_pos;
				self.cur_pos += 4;
				match pseudo::PseudoInst::decode(inst, self.bytes) {
					Some((inst, len)) => {
						self.bytes = &self.bytes[len..];
						self.cur_pos += len as u32;
						Some((pos, Inst::Pseudo(inst)))
					},
					None => Some((pos, Inst::Basic(inst))),
				}
			},

			// If we don't have enough for a `u32` or we didn't manage to
			// parse an instruction, try to parse a directive
			None => match Directive::decode(self.cur_pos, self.bytes) {
				Some((directive, len)) => {
					self.bytes = &self.bytes[len..];
					let pos = self.cur_pos;
					self.cur_pos += len as u32;
					Some((pos, Inst::Directive(directive)))
				},
				None => None,
			},
		}
	}
}
