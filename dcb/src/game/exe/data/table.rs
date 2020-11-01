//! Data table
//!
//! This module defines the [`DataTable`] type, which
//! stores all data locations within the executable.
//!
//! Typically this data will be a mix of the known data,
//! available through [`DataTable::known`] and heuristically
//! discovered data through instruction references, available
//! through [`DataTable::search_instructions`].

// Modules
pub mod error;

// Exports
pub use error::GetKnownError;

// Imports
use super::{Data, DataKind};
use crate::{
	game::exe::{
		instruction::{Directive, PseudoInstruction},
		Instruction, Pos,
	},
	util::DiscardingSortedMergeIter,
};
use std::{collections::BTreeSet, convert::TryInto, fs::File, iter::FromIterator};

/// Data table
///
/// Stores all data locations sorted by their address.
/// Data locations may be 'specialized', that is, a large data
/// location may have several smaller data locations inside of it,
/// as long as they only belong to the larger data location.
#[derive(PartialEq, Eq, Clone, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct DataTable(BTreeSet<Data>);

impl FromIterator<Data> for DataTable {
	fn from_iter<T: IntoIterator<Item = Data>>(iter: T) -> Self {
		Self(iter.into_iter().collect())
	}
}

impl DataTable {
	/// Merges two data tables, discarding duplicates from `other`.
	///
	/// This can be useful when combining known data locations and heuristically
	/// discovered data locations, as the known functions are always kept, and the
	/// duplicate discovered ones are discarded.
	#[must_use]
	pub fn merge(self, other: Self) -> Self {
		// Note: We don't return the iterator, as we want the user to
		//       keep the guarantees supplied by this type.
		DiscardingSortedMergeIter::new(self.0.into_iter(), other.0.into_iter()).collect()
	}

	/// Retrieves the smallest data location containing `pos`
	#[must_use]
	pub fn get(&self, pos: Pos) -> Option<&Data> {
		// Find the closest one and check if it contains `pos`
		// Note: We search from the end to make sure we grab the
		//       smaller locations first.
		self.0.range(..=pos).next_back().filter(|data| pos <= data.end_pos())
	}
}

impl DataTable {
	/// Returns all known data locations
	pub fn get_known() -> Result<Self, GetKnownError> {
		let file = File::open("resources/known_data.yaml").map_err(GetKnownError::File)?;

		serde_yaml::from_reader(file).map_err(GetKnownError::Parse)
	}

	/// Searches all instructions for references to
	/// executable data using certain heuristics.
	#[must_use]
	pub fn search_instructions<'a>(instructions: impl Iterator<Item = (Pos, &'a Instruction)> + Clone) -> Self {
		// Get all possible references to data
		let data_references: BTreeSet<Pos> = instructions
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

		// Then filter the instructions for data locations.
		instructions
			// Filter all non-directives
			.filter_map(|(pos, instruction)| match instruction {
				Instruction::Directive(directive) if data_references.contains(&pos) => Some((pos, directive)),
				_ => None,
			})
			.zip(0..)
			.map(|((pos, directive), idx)| {
				match directive {
					Directive::Ascii(ascii) => Data {
						name: format!("string_{idx}"),
						desc: String::new(),
						pos,
						kind: DataKind::AsciiStr { len: ascii.len().try_into().expect("String length didn't fit into a `u32`") },
					},
					Directive::Dw(_) => Data {
						name: format!("data_w{idx}"),
						desc: String::new(),
						pos,
						kind: DataKind::Word,
					},
					Directive::Dh(_) => Data {
						name: format!("data_h{idx}"),
						desc: String::new(),
						pos,
						kind: DataKind::HalfWord,
					},
					Directive::Db(_) => Data {
						name: format!("data_b{idx}"),
						desc: String::new(),
						pos,
						kind: DataKind::Byte,
					},
				}
			})
			.collect()
	}
}
