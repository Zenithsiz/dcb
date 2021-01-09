//! Executable iterator

// Imports
use super::{
	inst::{Inst, ParseIter},
	Data, Func,
};
use crate::{Exe, Pos};

/// Iterator over executable parts
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
			cur_pos: Exe::START_MEM_ADDRESS,
		}
	}
}

/// An executable item
pub enum ExeItem<'a> {
	/// A function
	Func(&'a Func),

	/// A data
	Data(&'a Data),

	/// Instruction
	Inst(Pos, Inst),
}

impl<'a> Iterator for Iter<'a> {
	type Item = ExeItem<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		// If we're at the end, return `None`
		if self.cur_pos == Exe::END_MEM_ADDRESS {
			return None;
		}

		// Try to get data
		// TODO: Not skip over small data somehow? Maybe just remove the ability to have overlapping data sections
		if let Some(data) = self.exe.data_table.get(self.cur_pos) {
			self.cur_pos = data.end_pos();
			return Some(ExeItem::Data(data));
		}

		// Else try to get a function
		if let Some(func) = self.exe.func_table.get(self.cur_pos) {
			self.cur_pos = func.end_pos;
			return Some(ExeItem::Func(func));
		}

		// Else simply get an instruction
		#[allow(clippy::indexing_slicing)] // Can't panic, as `cur_pos < END`
		let mut iter = ParseIter::new(&self.exe.bytes[self.cur_pos.as_mem_idx()..], self.cur_pos);
		let (pos, inst) = iter.next().expect("Iterator was empty before code ending");
		self.cur_pos = iter.cur_pos();
		Some(ExeItem::Inst(pos, inst))
	}
}
