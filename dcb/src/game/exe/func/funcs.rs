//! Function lists

// Imports
use super::{Func, WithInstructionsIter};
use crate::{
	game::exe::{
		instruction::{Directive, Register, SimpleInstruction},
		Instruction, Pos,
	},
	util::merge_iter::MergeSortedIter,
};
use maplit::hashmap;
use std::{collections::BTreeSet, iter::FromIterator, vec};

/// A sorted list of functions by their start address.
pub struct Funcs<S: AsRef<str>>(Vec<Func<S>>);

impl<S: AsRef<str>> FromIterator<Func<S>> for Funcs<S> {
	fn from_iter<T: IntoIterator<Item = Func<S>>>(iter: T) -> Self {
		Self(iter.into_iter().collect())
	}
}

impl<S: AsRef<str>> Funcs<S> {
	/// Merges two function lists, discarding any duplicates
	/// from `other`.
	#[must_use]
	pub fn merge(self, other: Self) -> MergeSortedIter<Func<S>, vec::IntoIter<Func<S>>, vec::IntoIter<Func<S>>> {
		MergeSortedIter::new(self.0.into_iter(), other.0.into_iter())
	}

	/// Adapts an instruction iterator to extract the current function
	pub fn with_instructions<'a, I: Iterator<Item = (Pos, &'a Instruction)>>(&'a self, instructions: I) -> WithInstructionsIter<'a, S, I> {
		WithInstructionsIter::new(instructions, self)
	}

	/// Retrieves a function with start address `pos`
	#[must_use]
	pub fn get(&self, pos: Pos) -> Option<&Func<S>> {
		// Note: As we're sorted, we can binary search
		self.0
			.binary_search_by(|func| func.start_pos.cmp(&pos))
			.ok()
			.and_then(|idx| self.0.get(idx))
	}
}

#[allow(clippy::use_self)] // We're not using `Funcs<S>`, but `Funcs<String>`
impl<S: AsRef<str> + Into<String>> Funcs<S> {
	/// Converts all strings to `String`.
	#[must_use]
	pub fn into_string(self) -> Funcs<String> {
		Funcs(
			self.0
				.into_iter()
				.map(|func| Func {
					name:      func.name.into(),
					signature: func.signature.into(),
					desc:      func.desc.into(),
					comments:  func.comments.into_iter().map(|(pos, comment)| (pos, comment.into())).collect(),
					start_pos: func.start_pos,
					end_pos:   func.end_pos,
				})
				.collect(),
		)
	}
}


impl Funcs<&'static str> {
	/// Returns all known functions
	#[must_use]
	pub fn known() -> Self {
		let mut functions: Vec<_> = Func::known().collect();

		functions.sort_by(|lhs, rhs| lhs.start_pos.cmp(&rhs.start_pos));
		Self(functions)
	}
}

impl Funcs<String> {
	/// Creates a new list of functions from an iterator over instructions
	#[must_use]
	pub fn from_instructions<'a>(instructions: impl Iterator<Item = (Pos, &'a Instruction)> + Clone) -> Self {
		// Get all instruction offsets present, ignoring directives.
		let offsets: BTreeSet<Pos> = instructions
			.clone()
			.filter_map(|(pos, instruction)| match instruction {
				Instruction::Directive(_) => None,
				_ => Some(pos),
			})
			.collect();

		// Get all returns
		let returns: BTreeSet<Pos> = instructions
			.clone()
			.filter_map(|(pos, instruction)| match instruction {
				Instruction::Simple(SimpleInstruction::Jr { rs: Register::Ra }) => Some(pos),
				_ => None,
			})
			.collect();

		// Now get every function entrance from jumps and `dw`s.
		let function_entrances: BTreeSet<Pos> = instructions
			.filter_map(|(_, instruction)| match instruction {
				Instruction::Simple(SimpleInstruction::Jal { target }) => Some(*target),
				Instruction::Directive(Directive::Dw(target) | Directive::DwRepeated { value: target, .. }) => Some(Pos(*target)),
				_ => None,
			})
			.filter(|target| (Instruction::CODE_START..Instruction::CODE_END).contains(target) && offsets.contains(target))
			.collect();

		// Now combine the function entrances and exits.
		// Note: functions will be sorted, as
		let functions = function_entrances
			.iter()
			.zip(0..)
			.map(|(&target, idx)| Func {
				name:      format!("func_{idx}"),
				signature: "".to_string(),
				desc:      "".to_string(),
				comments:  hashmap![],
				start_pos: target,
				end_pos:   returns.range(target..).next().copied().unwrap_or(Pos(0xFFFFFFFF)),
			})
			.collect();

		Self(functions)
	}
}
