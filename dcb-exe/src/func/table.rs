//! Function table
//!
//! This module defines the [`FuncTable`] type, which
//! stores all function within the executable.
//!
//! Typically these functions will be a mix of the known function,
//! available through [`FuncTable::get_known`] and heuristically
//! discovered functions through inst references, available
//! through [`FuncTable::search_instructions`].

// Modules
pub mod error;

// Exports
pub use error::GetKnownError;

// Imports
use super::Func;
use crate::{
	inst::{basic, Directive, Inst, Register},
	DataTable, Pos,
};
use dcb_util::DiscardingSortedMergeIter;
use std::{
	collections::{BTreeMap, BTreeSet},
	fs::File,
	iter::FromIterator,
	ops::{Bound, Range, RangeBounds},
};

/// Function table
///
/// Stores all functions sorted by their address.
/// Also guarantees all functions are unique and non-overlapping.
#[derive(PartialEq, Eq, Clone, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct FuncTable(BTreeSet<Func>);

impl FuncTable {
	/// Merges two data tables, discarding duplicates from `other`.
	///
	/// This can be useful when combining known functions and heuristically
	/// discovered function, as the known functions are always kept, and the
	/// duplicate discovered ones are discarded.
	#[must_use]
	pub fn merge_with(self, other: Self) -> Self {
		// Note: We don't return the iterator, as we want the user to
		//       keep the guarantees supplied by this type.
		DiscardingSortedMergeIter::new(self.0.into_iter(), other.0.into_iter()).collect()
	}

	/// Retrieves the function containing `pos`
	#[must_use]
	pub fn get_containing(&self, pos: Pos) -> Option<&Func> {
		// Find the first data that includes `pos`.
		self.range(..=pos).find(|func| func.contains(pos))
	}

	/// Retrieves the function at `pos`
	#[must_use]
	pub fn get_starting_at(&self, pos: Pos) -> Option<&Func> {
		self.get_containing(pos).filter(|func| func.start_pos == pos)
	}

	/// Returns a range of functions
	#[must_use]
	pub fn range(&self, range: impl RangeBounds<Pos>) -> impl DoubleEndedIterator<Item = &Func> + Clone {
		self.0.range(range)
	}
}

impl FuncTable {
	/// Returns all known functions
	pub fn get_known() -> Result<Self, GetKnownError> {
		let file = File::open("resources/game_funcs.yaml").map_err(GetKnownError::File)?;

		serde_yaml::from_reader(file).map_err(GetKnownError::Parse)
	}

	/// Creates a new list of functions from an iterator over insts
	#[must_use]
	#[allow(clippy::too_many_lines)] // TODO: Refactor
	pub fn search_instructions<'a>(
		insts_range: Range<Pos>, insts: impl Iterator<Item = (Pos, Inst<'a>)> + Clone, known_func_table: &Self,
		data_table: &DataTable,
	) -> Self {
		// Get all returns
		let returns: BTreeSet<Pos> = insts
			.clone()
			.filter_map(|(pos, inst)| match inst {
				// `jr $ra`
				Inst::Basic(basic::Inst::Jmp(basic::jmp::Inst::Reg(basic::jmp::reg::Inst {
					target: Register::Ra,
					kind: basic::jmp::reg::Kind::Jump,
				}))) => Some(pos),
				_ => None,
			})
			.collect();

		// Get all possible tailcalls
		let tailcalls: BTreeSet<Pos> = insts
			.clone()
			.filter_map(|(pos, inst)| match inst {
				Inst::Basic(basic::Inst::Jmp(
					// `j`
					basic::jmp::Inst::Reg(basic::jmp::reg::Inst {
						kind: basic::jmp::reg::Kind::Jump,
						..
					}) |
					// `jr`
					basic::jmp::Inst::Imm(basic::jmp::imm::Inst {
						kind: basic::jmp::imm::Kind::Jump,
						..
					}),
				)) => Some(pos),
				_ => None,
			})
			.collect();

		// Get all labels
		let labels: BTreeSet<Pos> = insts
			.clone()
			.filter_map(|(pos, inst)| match inst {
				// `j`
				Inst::Basic(basic::Inst::Jmp(basic::jmp::Inst::Imm(
					inst @ basic::jmp::imm::Inst {
						kind: basic::jmp::imm::Kind::Jump,
						..
					},
				))) => Some(inst.target(pos)),
				// Conditional jumps
				Inst::Basic(basic::Inst::Cond(inst)) => Some(inst.target(pos)),
				_ => None,
			})
			.filter(|target| insts_range.contains(target))
			.collect();

		// Now check every `Jal` and `Dw` for possible function entrances
		let function_entries: BTreeSet<Pos> = insts
			.filter_map(|(pos, inst)| match inst {
				// `jar`
				Inst::Basic(basic::Inst::Jmp(basic::jmp::Inst::Imm(
					inst @ basic::jmp::imm::Inst {
						kind: basic::jmp::imm::Kind::JumpLink,
						..
					},
				))) if pos.0 % 4 == 0 => Some(inst.target(pos)),
				// `dw`
				Inst::Directive(Directive::Dw(address)) if address % 4 == 0 => Some(Pos(address)),
				_ => None,
			})
			.filter(|target| insts_range.contains(target))
			.filter(|&target| data_table.get_containing(target).is_none())
			.collect();

		let mut cur_funcs = BTreeSet::<Func>::new();
		for (idx, &func_pos) in function_entries.iter().enumerate() {
			// Try to get the end position from the returns
			// Note: +8 for return + inst after.
			let mut end_pos: Pos = returns.range(func_pos..).next().copied().unwrap_or(func_pos) + 8;

			// If there's a function in between us and the return, use the last tailcall instead
			if let Some(next_func_pos) = function_entries.range(func_pos + 4i32..end_pos).next() {
				end_pos = tailcalls
					.range(..next_func_pos)
					.next_back()
					.copied()
					.unwrap_or(func_pos) + 8i32;

				// If we got a tailcall before this function, just end it 2 insts
				if end_pos <= func_pos {
					end_pos = func_pos + 8i32;
				}
			}

			// If this function would intersect any other, skip this one.
			if cur_funcs
				.range(..=func_pos)
				.next_back()
				.map_or(false, |func| func.end_pos > func_pos) ||
				cur_funcs
					.range(func_pos..)
					.next()
					.map_or(false, |func| func.start_pos < end_pos) ||
				known_func_table
					.range(..=func_pos)
					.next_back()
					.map_or(false, |func| func.end_pos > func_pos) ||
				known_func_table
					.range(func_pos..)
					.next()
					.map_or(false, |func| func.start_pos < end_pos)
			{
				continue;
			}

			// Get all labels within this function
			// Note: We skip labels on the function location itself.
			let labels = labels
				.range((Bound::Excluded(func_pos), Bound::Excluded(end_pos)))
				.enumerate()
				.map(|(idx, &pos)| (pos, format!("{idx}")))
				.collect();

			let func = Func {
				name: format!("func_{idx}"),
				signature: "fn()".to_owned(),
				desc: String::new(),
				inline_comments: BTreeMap::new(),
				comments: BTreeMap::new(),
				labels,
				start_pos: func_pos,
				end_pos,
			};
			assert!(cur_funcs.insert(func));
		}

		cur_funcs.into_iter().collect()
	}
}

impl FromIterator<Func> for FuncTable {
	fn from_iter<T: IntoIterator<Item = Func>>(iter: T) -> Self {
		Self(iter.into_iter().collect())
	}
}
