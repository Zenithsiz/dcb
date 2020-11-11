//! Executable functions
//!
//! This module stores known functions
//! within the executable, as well as
//! info on them, represented by the [`Func`]
//! type.
//!
//! The full list of known function may
//! be found at [`Func::known`].

// Modules
pub mod table;

// Exports
pub use table::FuncTable;

// Imports
use crate::exe::Pos;
use std::{borrow::Borrow, collections::HashMap};

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

	/// Comments
	#[serde(default)]
	pub comments: HashMap<Pos, String>,

	/// Labels
	#[serde(default)]
	pub labels: HashMap<Pos, String>,

	/// Start position
	pub start_pos: Pos,

	/// End position (non-inclusive)
	pub end_pos: Pos,
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

/// Only the start position is hashed, just as in the [`PartialEq`] impl.
impl std::hash::Hash for Func {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.start_pos.hash(state);
	}
}

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
