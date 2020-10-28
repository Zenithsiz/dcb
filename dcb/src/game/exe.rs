//! Executable
//!
//! This module contains the executable portion of the game,
//! as well as tools to decompile and recompile it.

// Modules
pub mod data;
pub mod error;
pub mod func;
pub mod header;
pub mod instruction;
pub mod pos;

// Exports
pub use data::Data;
pub use error::DeserializeError;
pub use func::Func;
pub use header::Header;
pub use instruction::Instruction;
pub use pos::Pos;

// Imports
use crate::{io::address::Data as DataAddress, GameFile};
use dcb_bytes::{ByteArray, Bytes};
use std::{
	convert::TryFrom,
	io::{Read, Seek, Write},
};

/// The game executable
///
/// This type holds all of the executable code
/// of the game.
#[derive(PartialEq, Eq, Clone, Hash, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Exe {
	/// The executable header
	pub header: Header,

	/// All data
	pub data: Vec<u8>,
}

impl Exe {
	/// Start address of the executable
	const START_ADDRESS: DataAddress = DataAddress::from_u64(0x58b9000);
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

		let mut data = vec![0u8; usize::try_from(header.size).expect("Header size didn't fit into a `usize`")];
		file.read_exact(data.as_mut()).map_err(DeserializeError::ReadData)?;

		Ok(Self { header, data })
	}
}
