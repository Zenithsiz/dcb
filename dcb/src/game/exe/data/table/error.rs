//! Errors

/// Error for [`DataTable::get_known`](super::DataTable::get_known)
#[derive(Debug, thiserror::Error)]
pub enum GetKnownError {
	/// Unable to open file
	#[error("Unable to open file")]
	File(#[source] std::io::Error),

	/// Unable to parse file
	#[error("Unable to parse file")]
	Parse(#[source] serde_yaml::Error),
}
