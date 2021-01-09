//! Executable
//!
//! This module contains the executable portion of the game,
//! as well as tools to decompile and recompile it.

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
pub use func::Func;
pub use header::Header;
pub use pos::Pos;

// Imports
use dcb_bytes::{ByteArray, Bytes};
use dcb_io::GameFile;
use std::io::{Read, Seek, Write};

use self::func::FuncTable;

/// The game executable
#[derive(PartialEq, Eq, Clone, Debug)]
//#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::as_conversions)] // `SIZE` always fits
pub struct Exe {
	/// The executable header
	pub header: Header,

	/// All bytes within the exe
	pub bytes: Box<[u8; Self::SIZE as usize]>,

	/// The data table.
	pub data_table: DataTable,

	/// The function table.
	pub func_table: FuncTable,
}

impl Exe {
	/// End memory address of the executable
	pub const END_MEM_ADDRESS: Pos = Pos(Self::START_MEM_ADDRESS.0 + Self::SIZE);
	/// Size of the executable
	pub const SIZE: u32 = 0x68000;
	/// Start address of the executable
	pub const START_ADDRESS: dcb_io::Data = dcb_io::Data::from_u64(0x58b9000);
	/// Start memory address of the executable
	pub const START_MEM_ADDRESS: Pos = Pos(0x80010000);
}

impl Exe {
	/// Deserializes the card table from a game file
	pub fn deserialize<R: Read + Write + Seek>(file: &mut GameFile<R>) -> Result<Self, DeserializeError> {
		// Seek to the table
		file.seek(std::io::SeekFrom::Start(Self::START_ADDRESS.as_u64()))
			.map_err(DeserializeError::Seek)?;

		// Read header
		let mut header_bytes = [0u8; <<Header as Bytes>::ByteArray as ByteArray>::SIZE];
		file.read_exact(&mut header_bytes).map_err(DeserializeError::ReadHeader)?;
		let header = Header::from_bytes(&header_bytes).map_err(DeserializeError::ParseHeader)?;

		// Make sure the header size is the one we expect
		if header.size != Self::SIZE {
			return Err(DeserializeError::WrongDataSize { header: Box::new(header) });
		}

		// Read all of the bytes
		#[allow(clippy::as_conversions)] // `SIZE` always fits
		let mut bytes = Box::new([0u8; Self::SIZE as usize]);
		file.read_exact(bytes.as_mut()).map_err(DeserializeError::ReadData)?;

		// Parse all instructions
		let insts = inst::ParseIter::new(&*bytes, Self::START_MEM_ADDRESS);

		// Parse all data and code
		let known_data_table = DataTable::get_known().map_err(DeserializeError::KnownDataTable)?;
		let heuristics_data_table = DataTable::search_instructions(insts.clone());
		let data_table = known_data_table.merge_with(heuristics_data_table);

		let known_func_table = FuncTable::get_known().map_err(DeserializeError::KnownFuncTable)?;
		let heuristics_func_table = FuncTable::search_instructions(insts);
		let func_table = known_func_table.merge_with(heuristics_func_table);

		Ok(Self {
			header,
			bytes,
			data_table,
			func_table,
		})
	}

	/*
	/// Returns a parsing iterator for all instructions
	#[must_use]
	pub const fn parse_iter(&self) -> inst::ParseIter {
		inst::ParseIter::new(&*self.bytes, Self::START_MEM_ADDRESS)
	}

	/// Returns this executable's bytes
	#[must_use]
	pub const fn bytes(&self) -> &[u8] {
		&*self.bytes
	}
	*/

	/// Creates an iterator over this executable
	#[must_use]
	pub const fn iter(&self) -> iter::Iter {
		iter::Iter::new(self)
	}

	/// Returns the instruction at `pos`
	#[must_use]
	pub fn get(&self, pos: Pos) -> Option<inst::Inst> {
		inst::ParseIter::new(&*self.bytes, pos).next().map(|(_, inst)| inst)
	}
}
