//! Errors

// Imports
use super::node;

/// Error for [`DataTable::new`](super::DataTable::new)
#[derive(Debug, thiserror::Error)]
pub enum NewError {
	/// Unable to insert data
	#[error("Unable to extend all data into empty table")]
	Extend(#[source] ExtendError),
}

/// Error for [`DataTable::extend`](super::DataTable::extend)
#[derive(Debug, thiserror::Error)]
pub enum ExtendError {
	/// Unable to insert data
	#[error("Unable to insert data into table")]
	Insert(#[source] node::InsertError),
}
