//! Executable iterator

// Imports
use crate::{inst::DecodeIter, Data, ExeReader, Func, Pos};

/// Iterator over the executable's data locations, functions and others.
#[derive(Clone, Debug)]
pub struct Iter<'a> {
	/// Executable
	exe: &'a ExeReader,

	/// Current position
	cur_pos: Pos,
}

impl<'a> Iter<'a> {
	/// Creates a new iterator
	pub(super) const fn new(exe: &'a ExeReader) -> Self {
		Self {
			exe,
			cur_pos: exe.header.start_pos,
		}
	}
}

/// An executable's item
#[derive(Clone, Debug)]
pub enum ExeItem<'a> {
	/// A function
	Func {
		/// The function metadata
		func: &'a Func,

		/// The instructions for this function
		insts: DecodeIter<'a>,
	},

	/// A data
	Data {
		/// The data metadata
		data: &'a Data,

		/// The instructions for this data
		insts: DecodeIter<'a>,
	},

	/// Unknown
	Unknown {
		/// Instruction in this unknown section
		insts: DecodeIter<'a>,
	},
}

impl<'a> Iterator for Iter<'a> {
	type Item = ExeItem<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		// If we're at the end, return `None`
		let cur_pos = self.cur_pos;
		if cur_pos >= self.exe.insts_range().end {
			return None;
		}

		// Try to get data
		if let Some(data) = self.exe.data_table.get_starting_at(cur_pos) {
			// Check the next data for our next position that isn't equal to our current one
			let end_pos = match self.exe.data_table.get_next_from(cur_pos) {
				// If it ends before or at the end of this data, use it
				Some(next_data) if next_data.start_pos() <= data.end_pos() => next_data.start_pos(),

				// Else end at the end of this data
				_ => data.end_pos(),
			};
			self.cur_pos = end_pos;

			return Some(ExeItem::Data {
				data,
				insts: self.exe.parse_iter_from(cur_pos..end_pos),
			});
		}

		// Else try to get a function
		if let Some(func) = self.exe.func_table.get_starting_at(self.cur_pos) {
			self.cur_pos = func.end_pos;
			return Some(ExeItem::Func {
				func,
				insts: self.exe.parse_iter_from(cur_pos..func.end_pos),
			});
		}

		// Else return an iterator until the next data / function, or until end, if none or past the end.
		let next_data = self.exe.data_table().get_next_from(cur_pos);
		let next_func = self.exe.func_table().range(cur_pos..).next();

		let end_pos = match (next_data, next_func) {
			(Some(next_data), Some(next_func)) => match next_data.start_pos() < next_func.start_pos {
				true => next_data.start_pos(),
				false => next_func.start_pos,
			},
			(Some(next_data), None) => next_data.start_pos(),
			(None, Some(next_func)) => next_func.start_pos,
			(None, None) => self.exe.insts_range().end,
		};

		// Make sure to limit the end position
		let end_pos = end_pos.min(self.exe.insts_range().end);
		self.cur_pos = end_pos;


		Some(ExeItem::Unknown {
			insts: self.exe.parse_iter_from(cur_pos..end_pos),
		})
	}
}
