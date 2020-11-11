//! Executable data locations
//!
//! This module defines the [`Data`] type, which
//! is responsible for storing a location within
//! the executable that represents a certain data
//! type, with associated metadata, such as a name
//! and description.

// Modules
pub mod table;
pub mod ty;

// Exports
pub use table::DataTable;
pub use ty::DataType;

// Imports
use crate::exe::Pos;
use std::{borrow::Borrow, cmp::Ordering};

/// A data location.
///
/// Two data locations are considered equal if they
/// share the same position and kind.
///
/// Their relative
/// order is also firstly dependent on their position
/// and kind, although when 2 data locations share
/// the same position, the larger one will appear first
/// in the ordering. This is to implement 'specialized'
/// data locations, where one may have a certain data location
/// inside of another for a specific purpose.
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

	/// Data type
	pub ty: DataType,
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
		self.ty.size()
	}
}

impl Borrow<Pos> for Data {
	fn borrow(&self) -> &Pos {
		&self.pos
	}
}

impl PartialEq for Data {
	fn eq(&self, other: &Self) -> bool {
		self.pos.eq(&other.pos)
	}
}

impl Eq for Data {}

impl std::hash::Hash for Data {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.pos.hash(state);
	}
}

impl PartialOrd for Data {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		// Delegate to `eq` since we have a total order.
		Some(self.cmp(other))
	}
}

impl Ord for Data {
	fn cmp(&self, other: &Self) -> Ordering {
		match self.pos.cmp(&other.pos) {
			// Note: We reverse so the larger size will be first.
			Ordering::Equal => self.size().cmp(&other.size()).reverse(),
			cmp => cmp,
		}
	}
}
