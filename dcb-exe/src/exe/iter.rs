//! Executable iterator

// Imports
use super::{inst::ParseIter, Data, Func};
use crate::{Exe, Pos};

/// Iterator over executable parts
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Iter<'a> {
	/// Executable
	exe: &'a Exe,

	/// Current position
	cur_pos: Pos,
}

impl<'a> Iter<'a> {
	/// Creates a new executable iterator
	pub(crate) const fn new(exe: &'a Exe) -> Self {
		Self {
			exe,
			cur_pos: Exe::MEM_START_ADDRESS,
		}
	}
}

/// An executable item
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum ExeItem<'a> {
	/// A function
	Func {
		/// The function metadata
		func: &'a Func,

		/// The instructions for this function
		insts: ParseIter<'a>,
	},

	/// A data
	Data {
		/// The data metadata
		data: &'a Data,

		/// The instructions for this data
		insts: ParseIter<'a>,
	},

	/// Unknown
	Unknown {
		/// Instruction in this unknown section
		insts: ParseIter<'a>,
	},
}

impl<'a> Iterator for Iter<'a> {
	type Item = ExeItem<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		// If we're at the end, return `None`
		let cur_pos = self.cur_pos;
		if cur_pos >= Exe::MEM_END_ADDRESS {
			return None;
		}

		// Try to get data
		if let Some(data) = self.exe.data_table.get(cur_pos) {
			// Check the next data for our next position that isn't equal to our current one
			let end_pos = match self.exe.data_table.range(cur_pos..).find(|next_data| next_data.pos != data.pos) {
				// If it ends before or at the end of this data, use it
				Some(next_data) if next_data.pos <= data.end_pos() => next_data.pos,

				// Else end at the end of this data
				_ => data.end_pos(),
			};
			self.cur_pos = end_pos;

			return Some(ExeItem::Data {
				data,
				insts: ParseIter::new(&self.exe.bytes[cur_pos.as_mem_idx()..end_pos.as_mem_idx()], cur_pos),
			});
		}

		// Else try to get a function
		if let Some(func) = self.exe.func_table.get(self.cur_pos) {
			self.cur_pos = func.end_pos;
			return Some(ExeItem::Func {
				func,
				insts: ParseIter::new(&self.exe.bytes[cur_pos.as_mem_idx()..func.end_pos.as_mem_idx()], cur_pos),
			});
		}

		// Else return an iterator until the next data / function, or until end, if none or past the end.
		let next_data = self.exe.data_table.range(cur_pos..).next();
		let next_func = self.exe.func_table.range(cur_pos..).next();

		let end_pos = match (next_data, next_func) {
			(Some(next_data), Some(next_func)) => match next_data.pos < next_func.start_pos {
				true => next_data.pos,
				false => next_func.start_pos,
			},
			(Some(next_data), None) => next_data.pos,
			(None, Some(next_func)) => next_func.start_pos,
			(None, None) => Exe::MEM_END_ADDRESS,
		};

		// Make sure to limit the end position
		let end_pos = end_pos.min(Exe::MEM_END_ADDRESS);
		self.cur_pos = end_pos;


		Some(ExeItem::Unknown {
			insts: ParseIter::new(&self.exe.bytes[cur_pos.as_mem_idx()..end_pos.as_mem_idx()], cur_pos),
		})
	}
}
