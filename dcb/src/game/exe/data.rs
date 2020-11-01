//! Executable data locations
//!
//! This module stores known data locations
//! within the executable, as well as info on
//! them, provided by the [`Data`] type.
//!
//! The full list of known data locations may
//! be found at [`Data::known`].

// Modules
pub mod kind;
pub mod table;

// Exports
pub use kind::DataKind;
pub use table::DataTable;

// Imports
use crate::game::exe::Pos;
use std::borrow::Borrow;

/// Data location
#[derive(Clone, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Data {
	/// Name
	pub name: String,

	/// Description
	#[serde(default)]
	pub desc: String,

	/// Start position
	pub pos: Pos,

	/// Data kind
	pub kind: DataKind,
}

impl Data {
	/// Returns the end position of this data
	#[must_use]
	pub fn end_pos(&self) -> Pos {
		self.pos + self.kind.size()
	}
}

impl Borrow<Pos> for Data {
	fn borrow(&self) -> &Pos {
		&self.pos
	}
}

/// Two data locations are equal if their position is the same.
impl PartialEq for Data {
	fn eq(&self, other: &Self) -> bool {
		self.pos.eq(&other.pos)
	}
}

impl Eq for Data {}

/// Only the position is hashed, just as in the [`PartialEq`] impl.
impl std::hash::Hash for Data {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.pos.hash(state);
	}
}

/// Only the position matters for the order
impl PartialOrd for Data {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		// Delegate to `eq` since we have a total order.
		Some(self.cmp(other))
	}
}

/// Only the position matters for the order
impl Ord for Data {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		// Only compare the start position
		self.pos.cmp(&other.pos)
	}
}
