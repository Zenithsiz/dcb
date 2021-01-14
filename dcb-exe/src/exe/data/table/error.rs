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

/// Error for [`DataTable::get_known`](super::DataTable::get_known)
#[derive(Debug, thiserror::Error)]
pub enum GetKnownError {
	/// Unable to open file
	#[error("Unable to open file")]
	File(#[source] std::io::Error),

	/// Unable to parse file
	#[error("Unable to parse file")]
	Parse(#[source] serde_yaml::Error),

	/// Unable to construct data table
	#[error("Unable to construct data table")]
	New(#[source] NewError),
}
