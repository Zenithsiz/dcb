//! Parsing iterator

// Imports
use super::{Inst, InstSize};
use crate::Pos;

/// Parsing iterator.
///
/// Parses instructions from a byte slice, `[u8]` along with it's position.
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
	type Item = (Pos, Inst<'a>);

	fn next(&mut self) -> Option<Self::Item> {
		// Try to read an instruction
		let inst = Inst::decode(self.cur_pos, self.bytes)?;
		let pos = self.cur_pos;

		// Then skip it in our bytes
		let len = inst.size();
		self.cur_pos += len;
		self.bytes = &self.bytes[len..];

		// And return it
		Some((pos, inst))
	}
}
