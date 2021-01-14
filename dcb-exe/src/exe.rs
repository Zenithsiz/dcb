//! Executable
//!
//! This module defines the `Exe` type, which encompasses the whole
//! executable part of the game file (`SLUS_013`).

// Modules
pub mod data;
pub mod error;
pub mod func;
pub mod header;
pub mod inst;
pub mod iter;
pub mod pos;

// Exports
pub use data::{Data, DataTable, DataType};
pub use error::DeserializeError;
pub use func::{Func, FuncTable};
pub use header::Header;
pub use pos::Pos;

// Imports
use dcb_bytes::{ByteArray, Bytes};
use dcb_io::GameFile;
use std::{
	convert::TryFrom,
	io::{Read, Seek, Write},
	ops::Range,
};

/// The game executable
#[derive(Clone, Debug)]
pub struct Exe {
	/// The executable header
	header: Header,

	/// All instruction bytes within the executable.
	bytes: Box<[u8]>,

	/// The data table.
	data_table: DataTable,

	/// The function table.
	func_table: FuncTable,
}

impl Exe {
	/// Start address of the executable in the game file.
	pub const START_ADDRESS: dcb_io::Data = dcb_io::Data::from_u64(0x58b9000);
}

impl Exe {
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

impl Exe {
	/// Deserializes the executable from a game file
	pub fn deserialize<R: Read + Write + Seek>(file: &mut GameFile<R>) -> Result<Self, DeserializeError> {
		// Seek to the table
		file.seek(std::io::SeekFrom::Start(Self::START_ADDRESS.as_u64()))
			.map_err(DeserializeError::Seek)?;

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

		// Read the known data and func table
		let known_data_table = DataTable::get_known().map_err(DeserializeError::KnownDataTable)?;
		let known_func_table = FuncTable::get_known().map_err(DeserializeError::KnownFuncTable)?;

		// Parse all instructions
		let insts = inst::ParseIter::new(&*bytes, &known_data_table, &known_func_table, header.start_pos);

		// Then parse all heuristic tables
		let heuristics_data = Data::search_instructions(insts_range.clone(), insts.clone());
		let heuristics_func_table = FuncTable::search_instructions(insts_range, insts, &known_data_table);
		let data_table = known_data_table.extend(heuristics_data).map_err(DeserializeError::MergeDataHeuristics)?;
		let func_table = known_func_table.merge_with(heuristics_func_table);

		Ok(Self {
			header,
			bytes,
			data_table,
			func_table,
		})
	}
}
