//! Parsing iterator

// Imports
use super::{
	basic::{self, Decodable as _},
	pseudo::{self, Decodable as _},
	Directive, Inst,
};
use crate::Pos;
use std::convert::TryFrom;

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

	/// Returns the current position of the iterator
	#[must_use]
	pub const fn cur_pos(&self) -> Pos {
		self.cur_pos
	}
}

impl<'a> Iterator for ParseIter<'a> {
	type Item = (Pos, Inst);

	#[allow(clippy::as_conversions, clippy::cast_possible_truncation)] // Byte lengths will always fit into a `u32`, as `self.bytes.len()` is always smaller than `u32`.
	fn next(&mut self) -> Option<Self::Item> {
		// If we're outside of code range, decode a directive
		if !Inst::CODE_RANGE.contains(&self.cur_pos) {
			let (directive, len) = Directive::decode(self.cur_pos, self.bytes)?;
			self.bytes = &self.bytes[len..];
			let pos = self.cur_pos;
			self.cur_pos += len as u32;
			return Some((pos, Inst::Directive(directive)));
		}

		// Else make the instruction iterator
		// Note: We fuse it to make sure that pseudo instructions don't try to skip
		//       invalid instructions.
		let mut insts = self
			.bytes
			.chunks(4)
			.map(|word| u32::from_ne_bytes([word[0], word[1], word[2], word[3]]))
			.map_while(|word| basic::Raw::from_u32(word).and_then(basic::Inst::decode))
			.fuse();

		// Try to decode a pseudo-instruction
		if let Some(inst) = pseudo::Inst::decode(insts.clone()) {
			let len = inst.size() * 4;
			self.bytes = &self.bytes[usize::try_from(len).expect("Instruction size didn't fit into a `usize`")..];
			let pos = self.cur_pos;
			self.cur_pos += len;
			return Some((pos, Inst::Pseudo(inst)));
		}

		// Else try to decode it as an basic instruction
		if let Some(inst) = insts.next() {
			self.bytes = &self.bytes[4..];
			let pos = self.cur_pos;
			self.cur_pos += 4;
			return Some((pos, Inst::Basic(inst)));
		}

		// Else read it as a directive
		match Directive::decode(self.cur_pos, self.bytes) {
			Some((directive, len)) => {
				self.bytes = &self.bytes[len..];
				let pos = self.cur_pos;
				self.cur_pos += len as u32;
				Some((pos, Inst::Directive(directive)))
			},
			None => None,
		}
	}
}
