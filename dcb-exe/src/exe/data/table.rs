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
use inst::directive::Directive;
use int_conv::SignExtended;

// Imports
use super::{Data, DataType};
use crate::exe::{
	inst::{self, basic, Inst},
	Pos,
};
use dcb_util::DiscardingSortedMergeIter;
use std::{collections::BTreeSet, fs::File, iter::FromIterator};

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

impl std::ops::Deref for DataTable {
	type Target = BTreeSet<Data>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DataTable {
	/// Merges two data tables, discarding duplicates from `other`.
	///
	/// This can be useful when combining known data locations and heuristically
	/// discovered data locations, as the known functions are always kept, and the
	/// duplicate discovered ones are discarded.
	#[must_use]
	pub fn merge_with(self, other: Self) -> Self {
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
		self.0.range(..=pos).next_back().filter(|data| pos < data.end_pos())
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
	pub fn search_instructions<'a>(insts: impl Iterator<Item = (Pos, Inst<'a>)> + Clone) -> Self {
		// Get all possible references to data
		let references: BTreeSet<Pos> = insts
			.clone()
			.filter_map(|(pos, inst)| match inst {
				Inst::Basic(basic::Inst::Load(basic::load::Inst { offset, .. }) | basic::Inst::Store(basic::store::Inst { offset, .. })) => {
					Some(pos + offset.sign_extended::<i32>())
				},
				/*
				Instruction::Pseudo(
					PseudoInst::La { target: offset, .. } |
					PseudoInst::Li32 { imm: offset, .. } |
					PseudoInst::LbImm { offset, .. } |
					PseudoInst::LbuImm { offset, .. } |
					PseudoInst::LhImm { offset, .. } |
					PseudoInst::LhuImm { offset, .. } |
					PseudoInst::LwlImm { offset, .. } |
					PseudoInst::LwImm { offset, .. } |
					PseudoInst::LwrImm { offset, .. } |
					PseudoInst::SbImm { offset, .. } |
					PseudoInst::ShImm { offset, .. } |
					PseudoInst::SwlImm { offset, .. } |
					PseudoInst::SwImm { offset, .. } |
					PseudoInst::SwrImm { offset, .. },
				) |
				*/
				Inst::Directive(Directive::Dw(address)) => Some(Pos(address)),
				_ => None,
			})
			.collect();

		// Then filter the instructions for data locations.
		insts
			// Filter all non-directives
			.filter_map(|(pos, instruction)| match instruction {
				Inst::Directive(directive) if references.contains(&pos) => Some((pos, directive)),
				_ => None,
			})
			.zip(0..)
			.map(|((pos, directive), idx)| {
				match directive {
					Directive::Ascii(string) => Data {
						name: format!("string_{idx}"),
						desc: String::new(),
						pos,
						ty: DataType::Array { ty: Box::new(DataType::AsciiChar), len: string.len() },
					},
					Directive::Dw(_) => Data {
						name: format!("data_w{idx}"),
						desc: String::new(),
						pos,
						ty: DataType::Word,
					},
					Directive::Dh(_) => Data {
						name: format!("data_h{idx}"),
						desc: String::new(),
						pos,
						ty: DataType::HalfWord,
					},
					Directive::Db(_) => Data {
						name: format!("data_b{idx}"),
						desc: String::new(),
						pos,
						ty: DataType::Byte,
					},
				}
			})
			.collect()
	}
}
