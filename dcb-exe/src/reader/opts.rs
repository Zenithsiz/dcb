//! Deserialization options

// Imports
use crate::{DataTable, FuncTable};

/// Options for deserialization
#[derive(Default, Debug)]
pub struct DeserializeOpts {
	/// Existing data table to use
	pub data_table: Option<DataTable>,

	/// Existing function table to use
	pub func_table: Option<FuncTable>,
}
