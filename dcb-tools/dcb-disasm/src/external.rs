//! External resources

// Imports
use crate::cli::CliData;
use dcb_exe::{data::DataKind, Data, DataTable, DataType, FuncTable, Pos};
use std::{collections::BTreeMap, str::FromStr};

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

		// Read all data
		let known_data: Vec<SerializedData> = zutil::parse_from_file(&known_data_path, serde_yaml::from_reader)
			.map_err(zutil::fmt_err_wrapper_owned)
			.map_err(|err| log::warn!("Unable to load game data from {known_data_path:?}: {err}"))
			.unwrap_or_default();
		let foreign_data: Vec<SerializedData> = zutil::parse_from_file(&foreign_data_path, serde_yaml::from_reader)
			.map_err(zutil::fmt_err_wrapper_owned)
			.map_err(|err| log::warn!("Unable to load foreign data from {foreign_data_path:?}: {err}"))
			.unwrap_or_default();

		// Bundle it up
		let all_data = known_data
			.into_iter()
			.map(|data| data.into_data(DataKind::Known))
			.chain(foreign_data.into_iter().map(|data| data.into_data(DataKind::Known)));

		// Then create the data table
		let data_table = all_data.fold(DataTable::default(), |mut data_table, data| {
			// Try to insert and log if we get an error.
			if let Err(err) = data_table.insert(data) {
				let data = err.data();
				log::warn!("Unable to add data {data}: {}", zutil::fmt_err_wrapper(&err));
			}
			data_table
		});

		let func_table = zutil::parse_from_file(&known_funcs_path, serde_yaml::from_reader)
			.map_err(zutil::fmt_err_wrapper_owned)
			.map_err(|err| log::warn!("Unable to load functions from {known_funcs_path:?}: {err}"))
			.unwrap_or_default();
		let inst_arg_overrides = zutil::parse_from_file(&inst_arg_overrides_path, serde_yaml::from_reader)
			.map_err(zutil::fmt_err_wrapper_owned)
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

/// Serialized game / foreign data
#[derive(Clone, Debug)]
#[derive(serde::Deserialize)]
pub struct SerializedData {
	/// Name
	name: String,

	/// Description
	#[serde(default)]
	desc: String,

	/// Start position
	pos: Pos,

	/// Data type
	#[serde(deserialize_with = "deserialize_data_ty")]
	ty: DataType,
}

impl SerializedData {
	/// Converts this data to a `Data` given it's kind
	pub fn into_data(self, kind: DataKind) -> Data {
		Data::new(self.name, self.desc, self.pos, self.ty, kind)
	}
}

fn deserialize_data_ty<'de, D>(deserializer: D) -> Result<DataType, D::Error>
where
	D: serde::Deserializer<'de>,
{
	let s = <String as serde::Deserialize>::deserialize(deserializer)?;

	DataType::from_str(&s).map_err(serde::de::Error::custom)
}
