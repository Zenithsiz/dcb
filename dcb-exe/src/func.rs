//! Executable functions
//!
//! This module stores known functions
//! within the executable, as well as
//! info on them, represented by the [`Func`]
//! type.

// Modules
pub mod error;
pub mod kind;
pub mod table;

// Exports
pub use error::ValidateError;
pub use kind::FuncKind;
pub use table::FuncTable;

// Imports
use crate::{
	inst::{basic, Directive, Inst, Register},
	DataTable, Pos,
};
use std::{
	borrow::Borrow,
	collections::{BTreeMap, BTreeSet},
	ops::{Bound, Range},
};

/// A function within the executable
#[derive(Clone, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Func {
	/// Function name
	pub name: String,

	/// Function signature
	#[serde(default)]
	pub signature: String,

	/// Description
	#[serde(default)]
	pub desc: String,

	/// Inline Comments
	#[serde(default)]
	pub inline_comments: BTreeMap<Pos, String>,

	/// Comments
	#[serde(default)]
	pub comments: BTreeMap<Pos, String>,

	/// Labels
	#[serde(default)]
	pub labels: BTreeMap<Pos, String>,

	/// Start position
	pub start_pos: Pos,

	/// End position (non-inclusive)
	pub end_pos: Pos,

	/// Kind
	pub kind: FuncKind,
}

// Getters
impl Func {
	/// Checks if this function contains `pos`
	#[must_use]
	pub fn contains(&self, pos: Pos) -> bool {
		(self.start_pos..self.end_pos).contains(&pos)
	}

	/// Validates this function
	pub fn validate(&self) -> Result<(), ValidateError<'_>> {
		// TODO: Validate name and signature?

		// If our positions don't make a proper range, return Err
		if self.end_pos < self.start_pos {
			return Err(ValidateError::InvalidRange {
				start_pos: self.start_pos,
				end_pos:   self.end_pos,
			});
		}

		// Check all positions of labels and comments are within our range.
		for (&pos, comment) in self.inline_comments.iter().chain(&self.comments) {
			if !self.contains(pos) {
				return Err(ValidateError::CommentPosOutOfBounds { pos, comment });
			}
		}
		for (&pos, label) in &self.labels {
			if !self.contains(pos) {
				return Err(ValidateError::LabelPosOutOfBounds { pos, label });
			}
		}

		Ok(())
	}
}

impl Func {
	/// Creates a new list of functions from an iterator over insts
	#[must_use]
	#[allow(clippy::too_many_lines)] // TODO: Refactor
	pub fn search_instructions<'a>(
		insts_range: Range<Pos>, insts: impl Iterator<Item = (Pos, Inst<'a>)> + Clone, func_table: Option<&FuncTable>,
		data_table: Option<&DataTable>,
	) -> BTreeSet<Self> {
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
			.filter(|&target| data_table.map_or(true, |data_table| data_table.get_containing(target).is_none()))
			.collect();

		let mut cur_funcs = BTreeSet::<Self>::new();
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
			let intersects = cur_funcs
				.range(..=func_pos)
				.next_back()
				.map_or(false, |func| func.end_pos > func_pos) ||
				cur_funcs
					.range(func_pos..)
					.next()
					.map_or(false, |func| func.start_pos < end_pos) ||
				func_table.map_or(false, |func_table| {
					func_table
						.range(..=func_pos)
						.next_back()
						.map_or(false, |func| func.end_pos > func_pos) ||
						func_table
							.range(func_pos..)
							.next()
							.map_or(false, |func| func.start_pos < end_pos)
				});
			if intersects {
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
				kind: FuncKind::Heuristics,
			};
			assert!(cur_funcs.insert(func));
		}

		cur_funcs
	}
}

impl Borrow<Pos> for Func {
	fn borrow(&self) -> &Pos {
		&self.start_pos
	}
}

/// Two functions are equal if their start position is the same.
impl PartialEq for Func {
	fn eq(&self, other: &Self) -> bool {
		self.start_pos.eq(&other.start_pos)
	}
}

impl Eq for Func {}


/// Only the start position matters for the order
impl PartialOrd for Func {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		// Delegate to `eq` since we have a total order.
		Some(self.cmp(other))
	}
}

/// Only the start position matters for the order
impl Ord for Func {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		// Only compare the start position
		self.start_pos.cmp(&other.start_pos)
	}
}
