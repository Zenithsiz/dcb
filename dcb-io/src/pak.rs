//! `.PAK` file parser

// Modules
pub mod entry;
pub mod error;
pub mod header;

// Exports
pub use entry::PakEntry;
pub use error::DeserializeError;
pub use header::Header;

// Imports
use dcb_bytes::Bytes;
use std::{convert::TryFrom, io};

/// A `.PAK` file
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct PakFile {
	/// All entries
	pub entries: Vec<PakEntry>,
}

impl PakFile {
	/// Deserializes a `.PAK` file from a reader
	#[allow(clippy::similar_names)] // Reader and header are different enough.
	pub fn deserialize<R: io::Read>(mut reader: R) -> Result<Self, DeserializeError> {
		// Keep reading headers until we find the final header.
		let mut entries = vec![];
		loop {
			// Read the header
			// Note: We do a two-part read so we can quit early if we find `0xffff`
			let mut header_bytes = [0u8; 0x8];
			reader.read_exact(&mut header_bytes[..0x4]).map_err(DeserializeError::ReadHeader)?;

			// If we found `0xFFFF`, this is the final header, return
			if header_bytes[..0x4] == [0xff, 0xff, 0xff, 0xff] {
				break;
			}

			// Then read the rest and parse the header
			reader.read_exact(&mut header_bytes[0x4..]).map_err(DeserializeError::ReadHeader)?;
			let header = Header::from_bytes(&header_bytes).map_err(DeserializeError::ParseHeader)?;

			// Read the data
			let size = usize::try_from(header.size).expect("`u32` didn't fit into a `usize`");
			let mut data = vec![0; size];
			reader.read_exact(&mut data).map_err(DeserializeError::ReadData)?;

			// Get the entry
			let entry = PakEntry::deserialize(header, data).map_err(DeserializeError::ParseEntry)?;
			entries.push(entry);
		}


		Ok(Self { entries })
	}
}
