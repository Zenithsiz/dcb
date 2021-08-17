//! External resources

// Imports
use crate::args::Args;
use dcb_exe::{data::DataKind, Data, DataTable, DataType, FuncTable, Pos};
use std::str::FromStr;

/// External resources
pub struct ExternalResources {
	/// Data table
	pub data_table: DataTable,

	/// Function table
	pub func_table: FuncTable,
}

impl ExternalResources {
	/// Loads external resources
	pub fn load(args: &Args) -> Self {
		// Read all data
		// TODO: Merge them both and just print the foreign data to the `.asm`.
		let game_data = match &args.game_data_path {
			Some(path) => match zutil::parse_from_file::<Vec<SerializedData>, _, _>(&path, serde_yaml::from_reader) {
				Ok(data) => data,
				Err(err) => {
					log::warn!("Unable to load game data from {}: {err}", path.display());
					Vec::new()
				},
			},
			None => Vec::new(),
		};
		let foreign_data = match &args.foreign_data_path {
			Some(path) => match zutil::parse_from_file::<Vec<SerializedData>, _, _>(&path, serde_yaml::from_reader) {
				Ok(data) => data,
				Err(err) => {
					log::warn!("Unable to load foreign data from {}: {err}", path.display());
					Vec::new()
				},
			},
			None => Vec::new(),
		};

		// Bundle it up
		let all_data = game_data
			.into_iter()
			.map(|data| data.into_data(DataKind::Known))
			.chain(foreign_data.into_iter().map(|data| data.into_data(DataKind::Foreign)));

		// Then create the data table
		let data_table = all_data.fold(DataTable::default(), |mut data_table, data| {
			// Try to insert and log if we get an error.
			if let Err(err) = data_table.insert(data) {
				let data = err.data();
				log::warn!("Unable to add data {data}: {}", zutil::fmt_err_wrapper(&err));
			}
			data_table
		});

		let func_table = match &args.game_funcs_path {
			Some(path) => match zutil::parse_from_file(&path, serde_yaml::from_reader) {
				Ok(data) => data,
				Err(err) => {
					log::warn!("Unable to load game functions from {}: {err}", path.display());
					FuncTable::new()
				},
			},
			None => FuncTable::new(),
		};

		Self { data_table, func_table }
	}
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
