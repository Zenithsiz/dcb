//! A `.PAK` entry

// Modules
pub mod error;

// Exports
pub use error::DeserializeError;

// Imports
use super::{header::Kind, Header};
use crate::tim::TimFile;
use dcb_util::{null_ascii_string::NullAsciiString, AsciiStrArr};
use std::convert::TryInto;

/// A `.PAK` entry
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum PakEntry {
	/// File header
	FileHeader {
		/// File name
		name: AsciiStrArr<0xb>,
	},

	/// Tim file
	TimFile(TimFile, Vec<u8>),

	/// Other
	Other(Kind),
}

impl PakEntry {
	/// Deserializes a `.PAK` file entry from it's header and ata
	pub fn deserialize(header: Header, data: Vec<u8>) -> Result<Self, DeserializeError> {
		let kind = header.file_kind;
		let entry = match kind {
			Kind::FileHeader => {
				let name_bytes: &[u8; 0xc] = data
					.get(..0xc)
					.and_then(|bytes| bytes.try_into().ok())
					.ok_or(DeserializeError::MissingName)?;
				let name = name_bytes.read_string().map_err(DeserializeError::ParseName)?;

				Self::FileHeader { name }
			},
			Kind::FileContents => {
				// Try to read it as a tim
				match TimFile::deserialize(std::io::Cursor::new(&data)) {
					Ok(tim) => Self::TimFile(tim, data),
					Err(_) => Self::Other(kind),
				}
			},
			_ => Self::Other(kind),
		};

		Ok(entry)
	}
}
