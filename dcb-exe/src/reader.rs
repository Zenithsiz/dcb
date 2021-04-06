//! Executable reader

// Modules
pub mod error;
pub mod iter;

// Exports
pub use error::{DeserializeError, GetKnownError};

// Imports
use crate::{inst, Data, DataTable, FuncTable, Header, Pos};
use dcb_bytes::{ByteArray, Bytes};
use std::{convert::TryFrom, io, ops::Range};

/// The game executable
#[derive(Clone, Debug)]
pub struct ExeReader {
	/// The executable header
	header: Header,

	/// All instruction bytes within the executable.
	bytes: Box<[u8]>,

	/// The data table.
	data_table: DataTable,

	/// The function table.
	func_table: FuncTable,
}

// Constants
impl ExeReader {
	/// MD5 Checksum
	pub const MD5_CHECKSUM: md5::Digest = md5::Digest(*b"\xc5\xf7\x5c\x43\xf4\xc5\x16\xcb\x4c\xc9\x11\x89\xfa\x76\xd7\x8a");
}

impl ExeReader {
	/// Returns this executable's header
	#[must_use]
	pub const fn header(&self) -> &Header {
		&self.header
	}

	/// Returns this executable's bytes
	#[must_use]
	pub const fn bytes(&self) -> &[u8] {
		&*self.bytes
	}

	/// Returns this executable's data table
	#[must_use]
	pub const fn data_table(&self) -> &DataTable {
		&self.data_table
	}

	/// Returns this executable's func table
	#[must_use]
	pub const fn func_table(&self) -> &FuncTable {
		&self.func_table
	}

	/// Returns this executable's instruction range
	#[must_use]
	pub fn insts_range(&self) -> Range<Pos> {
		let start = self.header.start_pos;
		let end = self.header.start_pos + self.header.size;
		start..end
	}

	/// Creates an iterator over this executable
	#[must_use]
	pub const fn iter(&self) -> iter::Iter {
		iter::Iter::new(self)
	}

	/// Returns a parsing iterator for all instructions
	#[must_use]
	pub fn parse_iter(&self) -> inst::ParseIter {
		self.parse_iter_from(self.insts_range())
	}

	/// Returns a parsing iterator starting from a range
	#[must_use]
	pub fn parse_iter_from(&self, range: Range<Pos>) -> inst::ParseIter {
		let start = range.start.offset_from(self.header.start_pos);
		let end = range.end.offset_from(self.header.start_pos);
		let bytes = &self.bytes[start..end];

		inst::ParseIter::new(bytes, &self.data_table, &self.func_table, range.start)
	}
}

impl ExeReader {
	/// Deserializes the executable from file
	pub fn deserialize<R: io::Read + io::Seek>(file: &mut R) -> Result<Self, DeserializeError> {
		// Read header
		let mut header_bytes = [0u8; <<Header as Bytes>::ByteArray as ByteArray>::SIZE];
		file.read_exact(&mut header_bytes).map_err(DeserializeError::ReadHeader)?;
		let header = Header::from_bytes(&header_bytes).map_err(DeserializeError::ParseHeader)?;

		// Get the instruction range
		let insts_range = {
			let start = header.start_pos;
			let end = header.start_pos + header.size;
			start..end
		};

		// Read all of the bytes
		let mut bytes = vec![0u8; usize::try_from(header.size).expect("Len didn't fit into `usize`")].into_boxed_slice();
		file.read_exact(bytes.as_mut()).map_err(DeserializeError::ReadData)?;

		// If it's checksum doesn't fit ours, return Err
		let checksum = md5::compute(&bytes);
		if checksum != Self::MD5_CHECKSUM {
			return Err(DeserializeError::DataChecksum { checksum });
		}

		// Read the known data and func table
		let mut known_data_table = self::get_known_data_table().map_err(DeserializeError::KnownDataTable)?;
		let known_func_table = FuncTable::get_known().map_err(DeserializeError::KnownFuncTable)?;

		// Parse all instructions
		let insts = inst::ParseIter::new(&*bytes, &known_data_table, &known_func_table, header.start_pos);

		// Then parse all heuristic tables
		let heuristics_data = Data::search_instructions(insts_range.clone(), insts.clone());
		let heuristics_func_table = FuncTable::search_instructions(insts_range, insts, &known_data_table);
		known_data_table.extend(heuristics_data);
		let func_table = known_func_table.merge_with(heuristics_func_table);

		Ok(Self {
			header,
			bytes,
			data_table: known_data_table,
			func_table,
		})
	}
}

/// Returns all known data locations
fn get_known_data_table() -> Result<DataTable, GetKnownError> {
	let game_data_file = std::fs::File::open("resources/game_data.yaml").map_err(GetKnownError::OpenGame)?;
	let game_data: Vec<Data> = serde_yaml::from_reader(game_data_file).map_err(GetKnownError::ParseGame)?;

	let foreign_data_file = std::fs::File::open("resources/foreign_data.yaml").map_err(GetKnownError::OpenForeign)?;
	let foreign_data: Vec<Data> = serde_yaml::from_reader(foreign_data_file).map_err(GetKnownError::ParseForeign)?;

	let mut data_table = DataTable::new(game_data);
	data_table.extend(foreign_data);

	Ok(data_table)
}
