//! Data table
//!
//! This module defines the [`DataTable`] type, which
//! stores all data locations within the executable.
//!
//! Typically this data will be a mix of the known data,
//! available through [`DataTable::known`] and heuristically
//! discovered data through instruction references, available
//! through [`DataTable::search_instructions`].

// Modules
pub mod error;
mod node;

// Exports
pub use error::{ExtendError, GetKnownError, NewError};

// Imports
use self::node::DataNode;
use super::Data;
use crate::exe::Pos;
use std::fs::File;

/// Data table
#[derive(Clone, Debug)]
pub struct DataTable {
	/// Root node
	///
	/// Note: The root data is never actually returned, it is a dummy data
	///       that encompasses all of the data positions.
	root: DataNode,
}

impl DataTable {
	/// Creates an empty data table
	#[must_use]
	pub fn empty() -> Self {
		let root = DataNode::new(Data::dummy());
		Self { root }
	}

	/// Creates a data table from an iterator of data
	pub fn new(data: impl IntoIterator<Item = Data>) -> Result<Self, NewError> {
		let table = Self::empty();
		table.extend(data).map_err(NewError::Extend)
	}

	/// Extends this data table with all values in `data`
	pub fn extend(mut self, data: impl IntoIterator<Item = Data>) -> Result<Self, ExtendError> {
		for data in data {
			self.root.insert(data).map_err(ExtendError::Insert)?;
		}

		Ok(self)
	}

	/// Retrieves the smallest data location containing `pos`
	#[must_use]
	pub fn get_containing(&self, pos: Pos) -> Option<&Data> {
		self.root.get_containing_deepest(pos).map(DataNode::data)
	}

	/// Retrieves the smallest data location at `pos`
	#[must_use]
	pub fn get_starting_at(&self, pos: Pos) -> Option<&Data> {
		self.get_containing(pos).filter(|data| data.pos == pos)
	}

	/// Returns the smallest data after `pos`
	#[must_use]
	pub fn get_next_from(&self, pos: Pos) -> Option<&Data> {
		// Keep doing down while there's a next node
		let mut cur_node = &self.root;
		let mut cur_next_node = None;
		while let Some(next_node) = cur_node.get_next_from(pos) {
			// Try to go deeper into the current node
			match cur_node.get_containing(pos) {
				// If we can go deeper, save the next node and try deeper
				Some(node) => {
					cur_next_node = Some(next_node);
					cur_node = node;
				},

				// If we can't go any deeper, go as deep as we can at the start of `next_node`
				None => return Some(next_node.get_containing_deepest(next_node.data().start_pos()).unwrap_or(next_node).data()),
			}
		}

		// If we got any next node, return it
		cur_next_node.map(DataNode::data)
	}
}

impl DataTable {
	/// Returns all known data locations
	pub fn get_known() -> Result<Self, GetKnownError> {
		let file = File::open("resources/known_data.yaml").map_err(GetKnownError::File)?;

		let data: Vec<Data> = serde_yaml::from_reader(file).map_err(GetKnownError::Parse)?;

		Self::new(data).map_err(GetKnownError::New)
	}
}
