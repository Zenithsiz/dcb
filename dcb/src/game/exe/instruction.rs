//! Psx cpu instructions

// Modules
pub mod directive;
pub mod pseudo;
pub mod raw;
pub mod reg;
pub mod simple;

// Exports
pub use directive::Directive;
pub use pseudo::PseudoInstruction;
pub use raw::{FromRawIter, Raw};
pub use reg::Register;
pub use simple::SimpleInstruction;

// Imports
use crate::game::exe::Pos;

/// An assembler instruction
#[derive(PartialEq, Eq, Clone, Debug)]
#[derive(derive_more::Display)]
pub enum Instruction {
	/// A simple instruction
	Simple(SimpleInstruction),

	/// A pseudo instruction
	Pseudo(PseudoInstruction),

	/// A directive
	Directive(Directive),
}

impl Instruction {
	/// End of the code itself in the executable.
	pub const CODE_END: Pos = Pos(0x8006dd3c);
	/// Start of the code itself in the executable.
	pub const CODE_START: Pos = Pos(0x80013e4c);
}

/// Iterator adaptor for converting [`RawInstruction`]s into [`Instruction`]s.
pub struct Iter<I: Iterator<Item = Raw> + Clone> {
	/// Underlying iterator
	iter: I,

	/// Remaining items from last iterator
	remaining: Option<Box<dyn Iterator<Item = (Pos, Instruction)>>>,
}

impl<I: Iterator<Item = Raw> + Clone> Iter<I> {
	/// Helper function to try to decode without consuming the iterator
	fn try_decode<T: FromRawIter>(iter: &I) -> (I, T::Decoded) {
		let mut cloned_iter = iter.clone();
		let decoded = T::decode(&mut cloned_iter);
		(cloned_iter, decoded)
	}

	/// Helper function to try to get instructions from `T`.
	fn try_next_from<T: FromRawIter + 'static>(&mut self, to_instruction: fn(T) -> Instruction) -> Option<(Pos, Instruction)> {
		// Try to decode it and get all instructions
		let (iter, instructions) = Self::try_decode::<T>(&self.iter);

		// Map the instructions to be an iterator over `Instruction` and peekable
		let mut instructions = instructions
			.into_iter()
			.map(move |(pos, decoded)| (pos, to_instruction(decoded)))
			.peekable();

		// Then check if we got any from the decode
		match instructions.next() {
			// If we did, set our iter, set any remaining instructions and return the instruction
			Some(instruction) => {
				self.iter = iter;
				// If there are any instructions left, set remaining, else just leave it
				if instructions.peek().is_some() {
					self.remaining = Some(Box::new(instructions));
				}
				Some(instruction)
			},

			// Else we didn't get anything, don't update the iterator.
			None => None,
		}
	}

	/// Returns the current position of the iterator
	fn cur_pos(&self) -> Option<Pos> {
		self.iter.clone().next().map(|raw| raw.pos)
	}
}

impl<I: Iterator<Item = Raw> + Clone> Iterator for Iter<I> {
	type Item = (Pos, Instruction);

	fn next(&mut self) -> Option<Self::Item> {
		// If we have remaining instruction, supply them
		if let Some(remaining) = self.remaining.as_mut() {
			if let Some(instruction) = remaining.next() {
				return Some(instruction);
			} else {
				// Note: We set it to none in case `next` is expensive to check.
				self.remaining = None;
			}
		}

		// Else get the current position
		let cur_pos = self.cur_pos()?;

		// If we're before the code start, just read directives
		if cur_pos < Instruction::CODE_START || cur_pos >= Instruction::CODE_END {
			return self.try_next_from(Instruction::Directive);
		}

		// Else try to decode it as a pseudo, simple or directive, in that order.
		self.try_next_from(Instruction::Pseudo)
			.or_else(|| self.try_next_from(Instruction::Simple))
			.or_else(|| self.try_next_from(Instruction::Directive))
	}
}

impl Instruction {
	/// Adapts an iterator over raw words to an instruction iterator
	pub fn new_iter<I: Iterator<Item = Raw> + Clone>(iter: I) -> Iter<I> {
		Iter { iter, remaining: None }
	}
}
