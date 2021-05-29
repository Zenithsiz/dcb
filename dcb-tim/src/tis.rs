//! `tim` Image collection

// Modules
pub mod error;

// Exports
pub use error::DeserializeError;

// Imports
use crate::Tim;
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{self, SeekFrom};

/// `tis` file
#[derive(PartialEq, Clone, Debug)]
pub struct Tis {
	/// All tims
	pub tims: Vec<Tim>,
}

impl Tis {
	/// Deserializes a tis file
	pub fn deserialize<R: io::Seek + io::Read>(reader: &mut R) -> Result<Self, DeserializeError> {
		// Read and validate the magic
		let magic = reader.read_u16::<LittleEndian>().map_err(DeserializeError::ReadMagic)?;
		if magic != 0x7054 {
			return Err(DeserializeError::InvalidMagic(magic));
		}

		// Then read the number of entries and all entries
		let entries_len = reader
			.read_u16::<LittleEndian>()
			.map_err(DeserializeError::ReadEntriesLen)?;
		let entries = (0..entries_len)
			.map(|_| reader.read_u32::<LittleEndian>())
			.collect::<Result<Vec<_>, _>>()
			.map_err(DeserializeError::ReadEntries)?;

		// Then read all tims
		let tims = entries
			.into_iter()
			.map(|idx| {
				// Seek to the position
				let pos = 4 * idx;
				reader
					.seek(SeekFrom::Start(u64::from(pos)))
					.map_err(DeserializeError::SeekTim)?;

				// Then deserialize it
				Tim::deserialize(reader).map_err(DeserializeError::DeserializeTim)
			})
			.collect::<Result<_, _>>()?;

		Ok(Self { tims })
	}
}
