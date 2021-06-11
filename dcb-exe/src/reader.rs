//! Executable reader.

// Modules
mod error;
pub mod iter;
pub mod opts;

// Exports
pub use error::DeserializeError;
pub use opts::DeserializeOpts;

// Imports
use crate::{inst, Data, DataTable, Func, FuncTable, Header, Pos};
use dcb_bytes::BytesReadExt;
use std::{convert::TryFrom, io, ops::Range};

/// Executable reader
///
/// Serves to read all information from the executable,
/// decode it and provide an interface to retrieve data
/// and functions, including their instructions.
#[derive(Clone, Debug)]
pub struct ExeReader {
	/// The executable header
	header: Header,

	/// All bytes of the executable (excluding header.)
	bytes: Box<[u8]>,

	/// Data table
	data_table: DataTable,

	/// Function table
	func_table: FuncTable,
}

// Constructors
impl ExeReader {
	/// Deserializes the executable from a file.
	///
	/// # Options
	/// Allows external data and function tables to be used during this deserialization.
	pub fn deserialize<R: io::Read + io::Seek>(file: &mut R, opts: DeserializeOpts) -> Result<Self, DeserializeError> {
		// Read header
		let header = file.read_deserialize::<Header>()?;

		// Read all of the bytes
		let mut bytes =
			vec![0u8; usize::try_from(header.size).expect("Len didn't fit into `usize`")].into_boxed_slice();
		file.read_exact(bytes.as_mut()).map_err(DeserializeError::ReadData)?;

		// Check if we were given any initial tables, else initialize them
		let mut data_table = opts.data_table.unwrap_or_else(DataTable::new);
		let mut func_table = opts.func_table.unwrap_or_else(FuncTable::new);

		// Then parse all heuristic tables
		let insts = inst::DecodeIter::new(&*bytes, &data_table, &func_table, header.start_pos);
		let insts_range = {
			let start = header.start_pos;
			let end = header.start_pos + header.size;
			start..end
		};
		let heuristics_data = Data::search_instructions(insts_range.clone(), insts.clone());
		let heuristics_func_table = Func::search_instructions(insts_range, insts, Some(&func_table), Some(&data_table));
		// Note: We ignore errors for when we can't insert heuristic data.
		for data in heuristics_data {
			#[allow(clippy::let_underscore_drop)] // We want to explicitly ignore it
			let _ = data_table.insert(data);
		}
		func_table.extend(heuristics_func_table);

		Ok(Self {
			header,
			bytes,
			data_table,
			func_table,
		})
	}
}

// Getters
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

	/// Returns the range of positions of this executable's
	/// instructions.
	#[must_use]
	pub fn insts_range(&self) -> Range<Pos> {
		let start = self.header.start_pos;
		let end = self.header.start_pos + self.header.size;
		start..end
	}

	/// Creates an iterator over this executable's data and functions.
	#[must_use]
	pub const fn iter(&self) -> iter::Iter {
		iter::Iter::new(self)
	}

	/// Returns an iterator that decodes instructions within a certain range.
	///
	/// # Panics
	/// Panics if `range` is not a valid range within this executable.
	#[must_use]
	pub fn decode_iter(&self, range: Range<Pos>) -> inst::DecodeIter {
		let start = range.start.offset_from(self.header.start_pos);
		let end = range.end.offset_from(self.header.start_pos);
		let bytes = &self.bytes[start..end];

		inst::DecodeIter::new(bytes, &self.data_table, &self.func_table, range.start)
	}
}
