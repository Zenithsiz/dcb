//! Directory entry reader

// Modules
pub mod error;

// Exports
pub use error::FromBytesError;


// Imports
use crate::{DirReader, FileReader};
use byteorder::{ByteOrder, LittleEndian};
use chrono::NaiveDateTime;
use dcb_util::{array_split, ascii_str_arr::AsciiChar, AsciiStrArr};

/// A directory entry reader kind
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum DirEntryReaderKind {
	/// A file
	File(FileReader),

	/// Directory
	Dir(DirReader),
}

/// A directory entry reader
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct DirEntryReader {
	/// Entry name
	name: AsciiStrArr<0x10>,

	/// Entry date
	date: NaiveDateTime,

	/// Entry kind
	kind: DirEntryReaderKind,
}

impl DirEntryReader {
	/// Reads a directory entry reader from bytes
	pub fn from_bytes(bytes: &[u8; 0x20]) -> Result<Option<Self>, FromBytesError> {
		let bytes = array_split!(bytes,
			kind      :  0x1,
			extension : [0x3],
			sector_pos: [0x4],
			size      : [0x4],
			data      : [0x4],
			name      : [0x10],
		);

		// Get the sector position
		let sector_pos = LittleEndian::read_u32(bytes.sector_pos);

		// Check kind
		let kind = match bytes.kind {
			0x0 => return Ok(None),
			0x1 => {
				// Read the extension and file size
				let mut extension = AsciiStrArr::from_bytes(bytes.extension).map_err(FromBytesError::Extension)?;
				extension.trim_end(AsciiChar::Null);
				let size = LittleEndian::read_u32(bytes.size);

				DirEntryReaderKind::File(FileReader::new(extension, sector_pos, size))
			},
			0x80 => DirEntryReaderKind::Dir(DirReader::new(sector_pos)),
			&kind => return Err(FromBytesError::InvalidKind(kind)),
		};

		// Special case some files which cause problems and return early, as if we encountered the final entry.
		#[allow(clippy::single_match)] // We might add more matches in the future
		match bytes.name {
			[0x83, 0x52, 0x83, 0x53, 0x81, 0x5B, 0x20, 0x81, 0x60, 0x20, 0x43, 0x41, 0x52, 0x44, 0x32, 0x00] => {
				log::warn!("Ignoring special entry: {:#x?}", bytes.name);
				return Err(FromBytesError::InvalidKind(0));
			},
			_ => (),
		}

		// Then get the name and date
		let mut name = AsciiStrArr::from_bytes(bytes.name).map_err(FromBytesError::Name)?;
		name.trim_end(AsciiChar::Null);
		let date = NaiveDateTime::from_timestamp(i64::from(LittleEndian::read_u32(bytes.data)), 0);

		Ok(Some(Self { name, date, kind }))
	}

	/// Returns this entry's name
	#[must_use]
	pub const fn name(&self) -> &AsciiStrArr<0x10> {
		&self.name
	}

	/// Returns this entry's date
	#[must_use]
	pub const fn date(&self) -> NaiveDateTime {
		self.date
	}

	/// Returns this entry's kind
	#[must_use]
	pub const fn kind(&self) -> &DirEntryReaderKind {
		&self.kind
	}
}
