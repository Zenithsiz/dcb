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
use crate::Pos;
use std::{collections::BTreeSet, iter::FromIterator, ops::RangeBounds};

/// Function table
///
/// Stores all functions sorted by their address.
/// Also guarantees all functions are unique and non-overlapping.
#[derive(PartialEq, Eq, Clone, Debug)]
#[derive(serde::Serialize)]
#[serde(transparent)]
pub struct FuncTable(BTreeSet<Func>);

// Constructors
impl FuncTable {
	/// Creates an empty function table
	#[must_use]
	pub const fn new() -> Self {
		Self(BTreeSet::new())
	}
}

// Getters
impl FuncTable {
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

// Note: `BTreeSet` already discards duplicates on it's own.
impl Extend<Func> for FuncTable {
	fn extend<T: IntoIterator<Item = Func>>(&mut self, funcs: T) {
		for func in funcs {
			self.extend_one(func);
		}
	}

	fn extend_one(&mut self, func: Func) {
		// Validate the function before inserting it.
		match func.validate() {
			Ok(()) => self.0.extend_one(func),
			// If it fails validation, discard it and warn
			Err(err) => log::warn!(
				"Function {} failed validation, ignoring:\n{}",
				func.name,
				dcb_util::fmt_err_wrapper(&err)
			),
		}
	}
}

impl FromIterator<Func> for FuncTable {
	fn from_iter<T: IntoIterator<Item = Func>>(iter: T) -> Self {
		let mut table = Self::new();
		table.extend(iter);
		table
	}
}

impl Default for FuncTable {
	fn default() -> Self {
		Self::new()
	}
}

impl<'de> serde::Deserialize<'de> for FuncTable {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		// Deserialize a vector of functions
		let funcs = Vec::<Func>::deserialize(deserializer)?;

		// Then collect into ourselves
		Ok(funcs.into_iter().collect())
	}
}
