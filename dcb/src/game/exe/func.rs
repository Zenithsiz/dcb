//! Executable functions

// Modules
pub mod iter;

// Exports
pub use iter::WithInstructionsIter;

// Imports
use crate::{
	game::exe::{
		instruction::{Directive, Register, SimpleInstruction},
		Instruction, Pos,
	},
	util::merge_iter::MergeSortedIter,
};
use maplit::hashmap;
use std::{
	collections::{BTreeSet, HashMap},
	iter::FromIterator,
	vec,
};

/// A function within the executable
#[derive(Clone, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Func<S: AsRef<str>> {
	/// Function name
	pub name: S,

	/// Function signature
	pub signature: S,

	/// Description
	pub desc: S,

	/// Comments
	pub comments: HashMap<Pos, S>,

	/// Start position
	pub start_pos: Pos,

	/// End position (non-inclusive)
	pub end_pos: Pos,
}

impl<S: AsRef<str>> PartialEq for Func<S> {
	fn eq(&self, other: &Self) -> bool {
		// Only compare the start position
		self.start_pos.eq(&other.start_pos)
	}
}

impl<S: AsRef<str>> Eq for Func<S> {}

impl<S: AsRef<str>> PartialOrd for Func<S> {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		// Delegate to `eq` since we have a total order.
		Some(self.cmp(other))
	}
}
impl<S: AsRef<str>> Ord for Func<S> {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		// Only compare the start position
		self.start_pos.cmp(&other.start_pos)
	}
}

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
		let mut functions = vec![
			Func {
				name:      "InitHeap",
				signature: "void(int* addr, unsigned int size)",
				desc:      "Calls A(0x39)",
				comments:  hashmap! {},
				start_pos: Pos(0x8006a734),
				end_pos:   Pos(0x8006a744),
			},
			Func {
				name:      "start",
				signature: "void(void)",
				desc:      "Executable start",
				comments:  hashmap! {
					Pos(0x80056280) => "Zero out 0x80077a08 .. 0x801ddf38 word by word.",
					Pos(0x80056284) => "^",
					Pos(0x80056288) => "^",
					Pos(0x8005628c) => "^",
					Pos(0x800562f8) => "InitHeap(0x8007f988, ???)",
					Pos(0x8005630c) => "func_1025(0x8007f98c)",
					Pos(0x80056324) => "func_1026(string_0, string_0)",
				},
				start_pos: Pos(0x80056270),
				end_pos:   Pos(0x80056330),
			},
			Func {
				name:      "func_1025",
				signature: "void(int*)",
				desc:      "",
				comments:  hashmap! {
					Pos(0x80013ef4) => "Called indefinitely?",
					Pos(0x80013efc) => "^ Due to this loop"
				},
				start_pos: Pos(0x80013e4c),
				end_pos:   Pos(0x80013f04),
			},
			Func {
				name:      "func_446",
				signature: "int(int)",
				desc:      "",
				comments:  hashmap! {},
				start_pos: Pos(0x80069124),
				end_pos:   Pos(0x80069150),
			},
		];

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
