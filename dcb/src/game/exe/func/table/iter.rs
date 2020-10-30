//! Iterators

// TODO: Deprecate in favor of a function + data iterator.

// Imports
use super::{Func, FuncTable};
use crate::game::exe::{Instruction, Pos};

/// Iterator of instructions along with the current function
pub struct WithInstructionsIter<'a, S: AsRef<str>, I: Iterator<Item = (Pos, &'a Instruction)>> {
	/// The instructions iterator
	instructions: I,

	/// All functions
	funcs: &'a FuncTable<S>,

	/// Current function
	cur_func: Option<&'a Func<S>>,
}

impl<'a, S: AsRef<str>, I: Iterator<Item = (Pos, &'a Instruction)>> WithInstructionsIter<'a, S, I> {
	/// Creates a new instructions iterator
	pub(super) fn new(instructions: I, funcs: &'a FuncTable<S>) -> Self {
		Self {
			instructions,
			funcs,
			cur_func: None,
		}
	}
}


impl<'a, S: AsRef<str>, I: Iterator<Item = (Pos, &'a Instruction)>> Iterator for WithInstructionsIter<'a, S, I> {
	type Item = (Pos, &'a Instruction, Option<&'a Func<S>>);

	fn next(&mut self) -> Option<Self::Item> {
		let (pos, instruction) = self.instructions.next()?;

		// If we're past the last instruction in the current function,
		// reset the instruction
		if let Some(cur_func) = self.cur_func {
			if cur_func.end_pos == pos {
				self.cur_func = None;
			}
		}

		// Else check if we have a current function
		match self.cur_func {
			// If we do, return it
			Some(cur_func) => Some((pos, instruction, Some(cur_func))),

			// Else check if we're at the start of a new function.
			None => match self.funcs.get(pos) {
				Some(cur_func) => {
					self.cur_func = Some(cur_func);
					Some((pos, instruction, Some(cur_func)))
				},
				None => Some((pos, instruction, None)),
			},
		}
	}
}
