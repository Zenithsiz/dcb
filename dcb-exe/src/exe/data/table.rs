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
	inst::{self, basic, pseudo, Inst},
	Pos,
};
use dcb_util::DiscardingSortedMergeIter;
use std::{
	collections::BTreeSet,
	fs::File,
	iter::FromIterator,
	ops::{Range, RangeBounds},
};

/// Data table
#[derive(PartialEq, Eq, Clone, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct DataTable(BTreeSet<Data>);

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
	pub fn get_containing(&self, pos: Pos) -> Option<&Data> {
		// Find the first data that includes `pos`.
		self.range(..=pos).filter(|data| data.contains(pos)).min_by_key(|data| data.size())
	}

	/// Retrieves the smallest data location at `pos`
	#[must_use]
	pub fn get_starting_at(&self, pos: Pos) -> Option<&Data> {
		self.get_containing(pos).filter(|data| data.pos == pos)
	}

	/// Returns a range of data
	#[must_use]
	pub fn range(&self, range: impl RangeBounds<Pos>) -> impl DoubleEndedIterator<Item = &Data> + Clone {
		self.0.range(range)
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
	pub fn search_instructions<'a>(insts_range: Range<Pos>, insts: impl Iterator<Item = (Pos, Inst<'a>)> + Clone) -> Self {
		// Get all possible references to data
		let references: BTreeSet<Pos> = insts
			.clone()
			.filter_map(|(pos, inst)| match inst {
				Inst::Basic(basic::Inst::Load(basic::load::Inst { offset, .. }) | basic::Inst::Store(basic::store::Inst { offset, .. })) => {
					Some(pos + offset.sign_extended::<i32>())
				},
				Inst::Pseudo(
					pseudo::Inst::LoadImm(pseudo::load_imm::Inst {
						kind: pseudo::load_imm::Kind::Address(Pos(address)) | pseudo::load_imm::Kind::Word(address),
						..
					}) |
					pseudo::Inst::Load(pseudo::load::Inst { target: Pos(address), .. }) |
					pseudo::Inst::Store(pseudo::store::Inst { target: Pos(address), .. }),
				) |
				Inst::Directive(Directive::Dw(address)) => Some(Pos(address)),
				_ => None,
			})
			.filter(|pos| insts_range.contains(pos))
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
						ty: DataType::AsciiStr { len: string.len() },
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

impl FromIterator<Data> for DataTable {
	fn from_iter<T: IntoIterator<Item = Data>>(iter: T) -> Self {
		Self(iter.into_iter().collect())
	}
}
