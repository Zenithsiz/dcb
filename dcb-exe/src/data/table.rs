//! Data table
//!
//! This module defines the [`DataTable`] type, responsible
//! for storing all data locations within the executable.
//! See it's type documentation for more information on how it works.

// Modules
mod error;
pub mod node;

// Exports
pub use error::InsertError;
pub use node::DataNode;

// Imports
use super::Data;
use crate::Pos;
use std::{collections::HashMap, fmt, rc::Rc};

/// Data table
///
/// The data locations are stored as a tree, where data that contains
/// other data has them has children, as such:
///
/// ```text
///    /-\       /-\      /----\
///    /---\   /---\  /-----------\
/// /-----------------------------\
/// ```
///
/// No two data locations can overlap and not be contained in another, in order
/// to be able to unambiguously get the smallest data location at a certain position.
/// There also may be no two data locations with the same position and type.
///
/// This tree structure allows getting any data location with a `ln(D)` complexity, `D`
/// being the depth of the tree, which usually does not go further than 4 or 5.
#[derive(Clone, Debug)]
pub struct DataTable {
	/// Root node
	///
	/// Note: The root data is never actually returned, it is a dummy data
	///       that encompasses all of the data positions.
	root: DataNode,

	/// Name to data for efficient name searching
	by_name: HashMap<String, Rc<Data>>,
}

// Constructors
impl DataTable {
	/// Creates an empty data table
	#[must_use]
	pub fn new() -> Self {
		let root = DataNode::new(Data::dummy());
		Self {
			root,
			by_name: HashMap::new(),
		}
	}
}

// Modifier
impl DataTable {
	/// Inserts data into this table
	pub fn insert(&mut self, data: Data) -> Result<(), InsertError> {
		// If some data with the same name exists, return Err
		if let Some(duplicate) = self.by_name.get(&data.name) {
			return Err(InsertError::DuplicateName {
				data,
				duplicate: Rc::clone(duplicate),
			});
		}

		// Else try to insert it into the root
		let data = self.root.insert(data).map_err(InsertError::Insert)?;

		// Then insert it into our names
		self.by_name.insert(data.name.clone(), data);

		Ok(())
	}
}

// Getters
impl DataTable {
	/// Retrieves the smallest data location containing `pos`
	#[must_use]
	pub fn get_containing(&self, pos: Pos) -> Option<&Data> {
		self.root.get_containing_deepest(pos).map(DataNode::data)
	}

	/// Retrieves the smallest data location starting at `pos`
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
				None => {
					return Some(
						next_node
							.get_containing_deepest(next_node.data().start_pos())
							.unwrap_or(next_node)
							.data(),
					)
				},
			}
		}

		// If we got any next node, return it
		cur_next_node.map(DataNode::data)
	}

	/// Searches for a data with name 'name'
	#[must_use]
	pub fn search_name(&self, name: &str) -> Option<&Data> {
		self.by_name.get(name).map(|data| &**data)
	}
}

impl Default for DataTable {
	fn default() -> Self {
		Self::new()
	}
}

impl fmt::Display for DataTable {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		for node in self.root.nodes() {
			write!(f, "{node}")?;
		}

		Ok(())
	}
}
