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
use std::{borrow::Borrow, cmp::Ordering};

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
		self.pos + self.size()
	}

	/// Returns the size, in bytes, of this data
	#[must_use]
	pub fn size(&self) -> u32 {
		self.kind.size()
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

/// Simply defers to the [`Ord`] impl.
impl PartialOrd for Data {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		// Delegate to `eq` since we have a total order.
		Some(self.cmp(other))
	}
}

/// The position is the main determining factor for each
/// data, but as 'specialized' data segments may exist,
/// when 2 data locations have the same position, the larger
/// one is put first.
impl Ord for Data {
	fn cmp(&self, other: &Self) -> Ordering {
		match self.pos.cmp(&other.pos) {
			// Note: We reverse so the larger size will be first.
			Ordering::Equal => self.size().cmp(&other.size()).reverse(),
			cmp => cmp,
		}
	}
}
