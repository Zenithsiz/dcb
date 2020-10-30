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
pub mod known;
pub mod table;

// Exports
pub use table::FuncTable;

// Imports
use crate::game::exe::Pos;
use std::{borrow::Borrow, collections::HashMap};

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

	/// Labels
	pub labels: HashMap<Pos, S>,

	/// Start position
	pub start_pos: Pos,

	/// End position (non-inclusive)
	pub end_pos: Pos,
}

#[allow(clippy::use_self)] // False positive
impl<S: AsRef<str> + Into<String>> Func<S> {
	/// Returns this function with owned `String`s.
	pub fn into_string(self) -> Func<String> {
		Func {
			name:      self.name.into(),
			signature: self.signature.into(),
			desc:      self.desc.into(),
			comments:  self.comments.into_iter().map(|(pos, comment)| (pos, comment.into())).collect(),
			labels:    self.labels.into_iter().map(|(pos, label)| (pos, label.into())).collect(),
			start_pos: self.start_pos,
			end_pos:   self.end_pos,
		}
	}
}

impl<S: AsRef<str>> Borrow<Pos> for Func<S> {
	fn borrow(&self) -> &Pos {
		&self.start_pos
	}
}

/// Two functions are equal if their start position is the same.
impl<S: AsRef<str>> PartialEq for Func<S> {
	fn eq(&self, other: &Self) -> bool {
		self.start_pos.eq(&other.start_pos)
	}
}

impl<S: AsRef<str>> Eq for Func<S> {}

/// Only the start position is hashed, just as in the [`PartialEq`] impl.
impl<S: AsRef<str>> std::hash::Hash for Func<S> {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.start_pos.hash(state);
	}
}

/// Only the start position matters for the order
impl<S: AsRef<str>> PartialOrd for Func<S> {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		// Delegate to `eq` since we have a total order.
		Some(self.cmp(other))
	}
}

/// Only the start position matters for the order
impl<S: AsRef<str>> Ord for Func<S> {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		// Only compare the start position
		self.start_pos.cmp(&other.start_pos)
	}
}
