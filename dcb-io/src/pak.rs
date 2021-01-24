//! `.PAK` file parser

// Modules
pub mod entry;
pub mod error;
pub mod header;

// Exports
pub use entry::PakEntry;
pub use error::FromReaderError;
pub use header::Header;

// Imports
use dcb_bytes::Bytes;
use std::io;

/// A `.PAK` file
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct PakFile {
	/// All entries
	entries: Vec<PakEntry>,
}

impl PakFile {
	/// Deserializes a `.PAK` file from a reader
	pub fn from_reader<R: io::Read + io::Seek>(reader: &mut R) -> Result<Self, FromReaderError> {
		// Keep reading headers until we find the final header.
		// TODO: Rewrite with scan + collect
		let mut entries = vec![];
		let mut cur_pos = 0;
		loop {
			// Read the header
			// Note: We do a two-part read so we can quit early if we find `0xffff`
			let mut header_bytes = [0u8; 0x8];
			reader.read_exact(&mut header_bytes[..0x4]).map_err(FromReaderError::ReadHeader)?;

			// If we found `0xFFFF`, this is the final header, return
			if header_bytes[..0x4] == [0xff, 0xff, 0xff, 0xff] {
				break;
			}

			// Then read the rest and parse the header
			reader.read_exact(&mut header_bytes[0x4..]).map_err(FromReaderError::ReadHeader)?;
			let header = Header::from_bytes(&header_bytes).map_err(FromReaderError::ParseHeader)?;
			cur_pos += 8;

			// Parse the entry
			let entry = PakEntry::new(header, cur_pos);
			entries.push(entry);

			// Then update our position and seek past
			let size = u64::from(header.size);
			cur_pos += size;
			reader.seek(io::SeekFrom::Start(cur_pos)).map_err(FromReaderError::SeekPastEntry)?;
		}

		Ok(Self { entries })
	}

	/// Returns all entries from this file
	#[must_use]
	pub fn entries(&self) -> &[PakEntry] {
		&self.entries
	}
}
