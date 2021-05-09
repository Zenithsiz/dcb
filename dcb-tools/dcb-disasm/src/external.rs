//! External resources

// Imports
use crate::cli::CliData;
use dcb_exe::{DataTable, FuncTable, Pos};
use std::collections::BTreeMap;

/// External resources
pub struct ExternalResources {
	/// Data table
	pub data_table: DataTable,

	/// Function table
	pub func_table: FuncTable,

	/// Overrides
	pub inst_arg_overrides: BTreeMap<ArgPos, String>,
}

impl ExternalResources {
	/// Loads external resources
	pub fn load(cli: &CliData) -> Self {
		let known_data_path = &cli.known_data_path;
		let foreign_data_path = &cli.foreign_data_path;
		let known_funcs_path = &cli.known_funcs_path;
		let inst_arg_overrides_path = &cli.inst_arg_overrides_path;

		let known_data: Vec<_> = dcb_util::parse_from_file(&known_data_path, serde_yaml::from_reader)
			.map_err(dcb_util::fmt_err_wrapper_owned)
			.map_err(|err| log::warn!("Unable to load game data from {known_data_path:?}: {err}"))
			.unwrap_or_default();
		let foreign_data: Vec<_> = dcb_util::parse_from_file(&foreign_data_path, serde_yaml::from_reader)
			.map_err(dcb_util::fmt_err_wrapper_owned)
			.map_err(|err| log::warn!("Unable to load foreign data from {foreign_data_path:?}: {err}"))
			.unwrap_or_default();
		let mut data_table = DataTable::default();
		for data in known_data.into_iter().chain(foreign_data) {
			// Try to insert and log if we get an error.
			if let Err(err) = data_table.insert(data) {
				let data = err.data();
				log::warn!("Unable to add data {data}: {}", dcb_util::fmt_err_wrapper(&err));
			}
		}

		let func_table = dcb_util::parse_from_file(&known_funcs_path, serde_yaml::from_reader)
			.map_err(dcb_util::fmt_err_wrapper_owned)
			.map_err(|err| log::warn!("Unable to load functions from {known_funcs_path:?}: {err}"))
			.unwrap_or_default();
		let inst_arg_overrides = dcb_util::parse_from_file(&inst_arg_overrides_path, serde_yaml::from_reader)
			.map_err(dcb_util::fmt_err_wrapper_owned)
			.map_err(|err| log::warn!("Unable to load instruction overrides from {inst_arg_overrides_path:?}: {err}"))
			.unwrap_or_default();

		Self {
			data_table,
			func_table,
			inst_arg_overrides,
		}
	}
}

/// Argument position
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct ArgPos {
	/// Position
	pub pos: Pos,

	/// Argument
	pub arg: usize,
}
