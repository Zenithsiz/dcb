//! Errors

// Imports
use super::node;
use crate::Data;
use std::rc::Rc;

/// Error for [`DataTable::insert`](super::DataTable::insert)
#[derive(Debug, thiserror::Error)]
pub enum InsertError {
	/// Cannot add data with duplicate name
	#[error("Cannot add data with duplicate name")]
	DuplicateName {
		/// The data that was trying to be inserted
		data: Data,

		/// The duplicate
		duplicate: Rc<Data>,
	},

	/// Cannot insert data into root node
	#[error("Unable to insert data into root node")]
	Insert(#[source] node::InsertError),
}

impl InsertError {
	/// Returns the data trying to be inserted
	#[must_use]
	pub fn data(&self) -> &Data {
		match self {
			InsertError::DuplicateName { data, .. } => data,
			InsertError::Insert(err) => err.data(),
		}
	}
}
