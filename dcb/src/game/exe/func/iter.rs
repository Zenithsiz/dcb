//! Iterators

// Imports
use super::{Func, Funcs};
use crate::game::exe::{
	instruction::{Register, SimpleInstruction},
	Instruction, Pos,
};

/// Iterator of instructions along with the current function
pub struct WithInstructionsIter<'a, S: AsRef<str>, I: Iterator<Item = (Pos, &'a Instruction)>> {
	/// The instructions iterator
	instructions: I,

	/// All functions
	funcs: &'a Funcs<S>,

	/// Last instruction
	last_instruction: Option<&'a Instruction>,

	/// Current function
	cur_func: Option<&'a Func<S>>,
}

impl<'a, S: AsRef<str>, I: Iterator<Item = (Pos, &'a Instruction)>> WithInstructionsIter<'a, S, I> {
	/// Creates a new instructions iterator
	pub(super) fn new(instructions: I, funcs: &'a Funcs<S>) -> Self {
		Self {
			instructions,
			funcs,
			last_instruction: None,
			cur_func: None,
		}
	}
}


impl<'a, S: AsRef<str>, I: Iterator<Item = (Pos, &'a Instruction)>> Iterator for WithInstructionsIter<'a, S, I> {
	type Item = (Pos, &'a Instruction, Option<&'a Func<S>>);

	fn next(&mut self) -> Option<Self::Item> {
		let (pos, instruction) = self.instructions.next()?;

		// Update our last instruction
		let last_instruction = self.last_instruction.replace(instruction);

		// Check if we had a return last instruction
		if let Some(Instruction::Simple(SimpleInstruction::Jr { rs: Register::Ra })) = last_instruction {
			// Set our cur function to `None` and return it
			let cur_func = self.cur_func.take();
			return Some((pos, instruction, cur_func));
		}

		// Else check if we have a current function
		match self.cur_func {
			// If we go, return it
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
