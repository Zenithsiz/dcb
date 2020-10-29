//! Data list

// Imports
use super::Data;
use crate::{
	game::exe::{
		instruction::{Directive, PseudoInstruction},
		Instruction, Pos,
	},
	util::merge_iter::MergeSortedIter,
};
use std::{
	collections::{btree_set, BTreeSet},
	iter::FromIterator,
};

/// List of data
pub struct AllData<S: AsRef<str>>(BTreeSet<Data<S>>);

impl<S: AsRef<str>> FromIterator<Data<S>> for AllData<S> {
	fn from_iter<T: IntoIterator<Item = Data<S>>>(iter: T) -> Self {
		Self(iter.into_iter().collect())
	}
}

impl<S: AsRef<str>> AllData<S> {
	/// Merges two function lists, discarding any duplicates
	/// from `other`.
	#[must_use]
	pub fn merge(self, other: Self) -> MergeSortedIter<Data<S>, btree_set::IntoIter<Data<S>>, btree_set::IntoIter<Data<S>>> {
		MergeSortedIter::new(self.0.into_iter(), other.0.into_iter())
	}

	/// Retrieves a data with start address `pos`
	#[must_use]
	pub fn get(&self, pos: Pos) -> Option<&Data<S>> {
		self.0.get(&pos)
	}
}

#[allow(clippy::use_self)] // We're not using `AllData<S>`, but `AllData<String>`
impl<S: AsRef<str> + Into<String>> AllData<S> {
	/// Converts all strings to `String`.
	#[must_use]
	pub fn into_string(self) -> AllData<String> {
		AllData(
			self.0
				.into_iter()
				.map(|data| match data {
					Data::Ascii { name, desc, start_pos } => Data::Ascii {
						name: name.into(),
						desc: desc.into(),
						start_pos,
					},
					Data::Bytes { name, desc, start_pos } => Data::Bytes {
						name: name.into(),
						desc: desc.into(),
						start_pos,
					},
				})
				.collect(),
		)
	}
}


impl AllData<&'static str> {
	/// Returns all known functions
	#[must_use]
	pub fn known() -> Self {
		Self(Data::known().collect())
	}
}


impl AllData<String> {
	/// Creates a new list of data from an iterator over instructions
	#[must_use]
	pub fn from_instructions<'a>(instructions: impl Iterator<Item = (Pos, &'a Instruction)> + Clone) -> Self {
		// Get all directive references
		let directive_references: BTreeSet<Pos> = instructions
			.clone()
			.filter_map(|(_, instruction)| match instruction {
				Instruction::Pseudo(
					PseudoInstruction::La { target: offset, .. } |
					PseudoInstruction::Li32 { imm: offset, .. } |
					PseudoInstruction::LbImm { offset, .. } |
					PseudoInstruction::LbuImm { offset, .. } |
					PseudoInstruction::LhImm { offset, .. } |
					PseudoInstruction::LhuImm { offset, .. } |
					PseudoInstruction::LwlImm { offset, .. } |
					PseudoInstruction::LwImm { offset, .. } |
					PseudoInstruction::LwrImm { offset, .. } |
					PseudoInstruction::SbImm { offset, .. } |
					PseudoInstruction::ShImm { offset, .. } |
					PseudoInstruction::SwlImm { offset, .. } |
					PseudoInstruction::SwImm { offset, .. } |
					PseudoInstruction::SwrImm { offset, .. },
				) |
				Instruction::Directive(Directive::Dw(offset)) => Some(Pos(*offset)),
				_ => None,
			})
			.collect();

		Self(
			instructions
				.filter_map(|(pos, instruction)| match instruction {
					Instruction::Directive(directive) if directive_references.contains(&pos) => Some((pos, directive)),
					_ => None,
				})
				.zip(0..)
				.map(|((pos, directive), idx)| match directive {
					Directive::Ascii(_) => Data::Ascii {
						name:      format!("string_{idx}"),
						desc:      "".to_string(),
						start_pos: pos,
					},

					Directive::Dw(_) => Data::Bytes {
						name:      format!("data_{idx}"),
						desc:      "".to_string(),
						start_pos: pos,
					},
				})
				.collect(),
		)
	}
}
