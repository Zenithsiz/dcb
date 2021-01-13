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
/// share the same position.
///
/// Their relative order first depends on their position.
/// When their positions are equal, the larger one will
/// appear first in the order.
/// This is to implement `specialized` data locations, where
/// a large data location can have multiple data locations inside.
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

	/// Checks if this data contains `pos`
	#[must_use]
	pub fn contains(&self, pos: Pos) -> bool {
		(self.pos..self.end_pos()).contains(&pos)
	}

	/// Returns the size, in bytes, of this data
	#[must_use]
	pub fn size(&self) -> usize {
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

impl PartialOrd for Data {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		// Delegate to `eq` since we have a total order.
		Some(self.cmp(other))
	}
}

impl Ord for Data {
	fn cmp(&self, other: &Self) -> Ordering {
		match self.pos.cmp(&other.pos) {
			// Note: If positions are equal, the smaller one comes first
			Ordering::Equal => self.size().cmp(&other.size()),
			cmp => cmp,
		}
	}
}
