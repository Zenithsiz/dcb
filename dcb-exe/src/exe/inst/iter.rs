//! Parsing iterator

// Imports
use super::{DecodeError, Inst, InstSize};
use crate::{
	exe::{DataTable, FuncTable},
	Pos,
};

/// Parsing iterator.
///
/// Parses instruction from a byte slice, along with it's memory position.
/// It also references the data and function table, force-decoding certain
/// instructions if inside a data location or function table.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct ParseIter<'a> {
	/// Remaining bytes
	bytes: &'a [u8],

	/// Starting position of bytes
	cur_pos: Pos,

	/// Data table
	data_table: &'a DataTable,

	/// Func table
	func_table: &'a FuncTable,
}

impl<'a> ParseIter<'a> {
	/// Creates a new parsing iterator
	#[must_use]
	pub const fn new(bytes: &'a [u8], data_table: &'a DataTable, func_table: &'a FuncTable, start_pos: Pos) -> Self {
		Self {
			bytes,
			cur_pos: start_pos,
			data_table,
			func_table,
		}
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
		let inst = match Inst::decode(self.cur_pos, self.bytes, self.data_table, self.func_table) {
			Ok(inst) => inst,
			Err(err) => match err {
				// If we're in an invalid data location, warn and skip it
				DecodeError::InvalidDataLocation { data } => {
					log::warn!(
						"Attempted to decode in position {} from within data location {}({})@{}",
						self.cur_pos,
						data.name,
						data.ty,
						data.pos
					);
					self.cur_pos = data.end_pos();
					return self.next(); // Tailcall
				},
				// If we're out of bytes, return `None`
				DecodeError::NoBytes => return None,
			},
		};
		let pos = self.cur_pos;

		// Then skip it in our bytes
		let len = inst.size();
		self.cur_pos += len;
		self.bytes = &self.bytes[len..];

		// And return it
		Some((pos, inst))
	}
}
