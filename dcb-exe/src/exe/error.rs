//! Errors

// Imports
use super::{data, func, header};

/// Error type for [`Table::deserialize`]
#[derive(Debug, thiserror::Error)]
pub enum DeserializeError {
	/// Unable to seek game file
	#[error("Unable to seek game file to executable")]
	Seek(#[source] std::io::Error),

	/// Unable to read header
	#[error("Unable to read header")]
	ReadHeader(#[source] std::io::Error),

	/// Unable to parse header
	#[error("Unable to parse header")]
	ParseHeader(#[source] header::FromBytesError),

	/// Unable to read data
	#[error("Unable to read data")]
	ReadData(#[source] std::io::Error),

	/// Unable to get known data
	#[error("Unable to get known data table")]
	KnownDataTable(#[source] GetKnownError),

	/// Unable to get known data
	#[error("Unable to get known func table")]
	KnownFuncTable(#[source] func::table::GetKnownError),

	/// Unable to merge heuristics
	#[error("Unable to merge heuristics")]
	MergeDataHeuristics(#[source] data::table::ExtendError),
}

/// Error type for getting the known function table
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
	New(#[source] data::table::NewError),
}
