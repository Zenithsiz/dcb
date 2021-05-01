//! Decoding iterator

// Imports
use super::{DecodeError, Inst, InstSize};
use crate::{DataTable, FuncTable, Pos};

/// Decoding iterator
///
/// Decodes instructions from a byte slice with an initial memory position.
/// References the data and function tables too.
#[derive(Clone, Debug)]
pub struct DecodeIter<'a> {
	/// Remaining bytes
	bytes: &'a [u8],

	/// Starting position of bytes
	cur_pos: Pos,

	/// Data table
	data_table: &'a DataTable,

	/// Func table
	func_table: &'a FuncTable,

	/// Previous instruction
	prev_inst: Option<Inst<'a>>,
}

impl<'a> DecodeIter<'a> {
	/// Creates a new decoding iterator
	#[must_use]
	pub const fn new(bytes: &'a [u8], data_table: &'a DataTable, func_table: &'a FuncTable, start_pos: Pos) -> Self {
		Self {
			bytes,
			cur_pos: start_pos,
			data_table,
			func_table,
			prev_inst: None,
		}
	}

	/// Returns the current position of the iterator
	#[must_use]
	pub const fn cur_pos(&self) -> Pos {
		self.cur_pos
	}
}

impl<'a> Iterator for DecodeIter<'a> {
	type Item = (Pos, Inst<'a>);

	fn next(&mut self) -> Option<Self::Item> {
		// Try to read an instruction
		let inst = match Inst::decode(
			self.cur_pos,
			self.bytes,
			self.data_table,
			self.func_table,
			self.prev_inst.as_ref(),
		) {
			Ok(inst) => inst,
			Err(err) => match err {
				// If we're in an invalid data location, panic
				DecodeError::InvalidDataLocation { data, err } => panic!(
					"Attempted to decode in position {} from within data location {data}:\n{}",
					self.cur_pos,
					dcb_util::DisplayWrapper::new(|f| dcb_util::fmt_err(&err, f)),
				),
				// If we're out of bytes, return `None`
				DecodeError::NoBytes => return None,
			},
		};
		let pos = self.cur_pos;

		// Then skip it in our bytes
		let len = inst.size();
		self.cur_pos += len;
		self.bytes = &self.bytes[len..];

		// Set our previous instruction
		self.prev_inst = Some(inst);

		// And return it
		Some((pos, inst))
	}
}
