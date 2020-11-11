//! Executable
//!
//! This module contains the executable portion of the game,
//! as well as tools to decompile and recompile it.

// Modules
pub mod data;
pub mod error;
//pub mod func;
pub mod header;
pub mod instruction;
pub mod pos;

// Exports
pub use data::{Data, DataTable, DataType};
pub use error::DeserializeError;
//pub use func::Func;
pub use header::Header;
pub use instruction::Instruction;
pub use pos::Pos;

// Imports
use dcb_bytes::{ByteArray, Bytes};
use dcb_io::GameFile;
use std::{
	convert::TryFrom,
	io::{Read, Seek, Write},
};

/// The game executable
#[derive(PartialEq, Eq, Clone, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Exe {
	/// The executable header
	pub header: Header,

	/// The data table.
	pub data_table: DataTable,
	//pub data: Vec<u8>,
}

impl Exe {
	/// Code range
	///
	/// Everything outside of this range will be considered data.
	pub const CODE_RANGE: std::ops::Range<Pos> = Pos(0x80013e4c)..Pos(0x8006dd3c);
	/// Start address of the executable
	const START_ADDRESS: dcb_io::Data = dcb_io::Data::from_u64(0x58b9000);
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

		//
		//file.bytes();

		let mut data = vec![0u8; usize::try_from(header.size).expect("Header size didn't fit into a `usize`")];
		file.read_exact(data.as_mut()).map_err(DeserializeError::ReadData)?;

		todo!();
		//Ok(Self { header, data })
	}
}
