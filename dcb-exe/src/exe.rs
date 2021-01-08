//! Executable
//!
//! This module contains the executable portion of the game,
//! as well as tools to decompile and recompile it.

// Modules
pub mod data;
pub mod error;
pub mod header;
pub mod inst;
pub mod pos;
//pub mod func;

// Exports
pub use data::{Data, DataTable, DataType};
pub use error::DeserializeError;
pub use header::Header;
pub use pos::Pos;
//pub use func::Func;

// Imports
use self::inst::Inst;
use dcb_bytes::{ByteArray, Bytes};
use dcb_io::GameFile;
use std::io::{Read, Seek, Write};

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
}

impl Exe {
	/// Size of the executable
	const SIZE: u32 = 0x68000;
	/// Start address of the executable
	const START_ADDRESS: dcb_io::Data = dcb_io::Data::from_u64(0x58b9000);
	/// Start memory address of the executable
	const START_MEM_ADDRESS: Pos = Pos(0x80010000);
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
		let heuristics_data_table = DataTable::search_instructions(insts);

		let data_table = known_data_table.merge_with(heuristics_data_table);

		Ok(Self { header, bytes, data_table })
	}

	/// Returns a parsing iterator for all instructions
	#[must_use]
	pub const fn parse_iter(&self) -> inst::ParseIter {
		inst::ParseIter::new(&*self.bytes, Self::START_MEM_ADDRESS)
	}
}
