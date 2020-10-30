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
pub mod known;
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
pub struct Data<S: AsRef<str>> {
	/// Name
	pub name: S,

	/// Description
	pub desc: S,

	/// Start position
	pub pos: Pos,

	/// Data kind
	pub kind: DataKind,
}

impl<S: AsRef<str>> Data<S> {
	/// Returns the end position of this data
	pub fn end_pos(&self) -> Pos {
		self.pos + self.kind.size()
	}
}

#[allow(clippy::use_self)] // False positive
impl<S: AsRef<str> + Into<String>> Data<S> {
	/// Returns this data with owned `String`s.
	pub fn into_string(self) -> Data<String> {
		Data {
			name: self.name.into(),
			desc: self.desc.into(),
			pos:  self.pos,
			kind: self.kind,
		}
	}
}

impl<S: AsRef<str>> Borrow<Pos> for Data<S> {
	fn borrow(&self) -> &Pos {
		&self.pos
	}
}

/// Two data locations are equal if their position is the same.
impl<S: AsRef<str>> PartialEq for Data<S> {
	fn eq(&self, other: &Self) -> bool {
		self.pos.eq(&other.pos)
	}
}

impl<S: AsRef<str>> Eq for Data<S> {}

/// Only the position is hashed, just as in the [`PartialEq`] impl.
impl<S: AsRef<str>> std::hash::Hash for Data<S> {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.pos.hash(state);
	}
}

/// Only the position matters for the order
impl<S: AsRef<str>> PartialOrd for Data<S> {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		// Delegate to `eq` since we have a total order.
		Some(self.cmp(other))
	}
}

/// Only the position matters for the order
impl<S: AsRef<str>> Ord for Data<S> {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		// Only compare the start position
		self.pos.cmp(&other.pos)
	}
}
