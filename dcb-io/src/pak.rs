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
	pub entries: Vec<PakEntry>,
}

impl PakFile {
	/// Deserializes a `.PAK` file from a reader
	#[allow(clippy::similar_names)] // Reader and header are different enough.
	pub fn from_reader<R: io::Read + io::Seek>(reader: &mut R) -> Result<Self, FromReaderError> {
		// Keep reading headers until we find the final header.
		let mut entries = vec![];
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


			// Parse the entry
			let start_pos = reader.stream_position().map_err(FromReaderError::GetStreamPos)?;
			let entry = PakEntry::from_reader(reader, header).map_err(FromReaderError::ParseEntry)?;
			entries.push(entry);

			// Make sure we seek to the end of the pak entry.
			reader
				.seek(io::SeekFrom::Start(start_pos + u64::from(header.size)))
				.map_err(FromReaderError::SetStreamPos)?;
		}


		Ok(Self { entries })
	}
}
