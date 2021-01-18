//! Directory entry

// Modules
pub mod error;

// Exports
use byteorder::{ByteOrder, LittleEndian};
pub use error::FromBytesError;

// Imports
use super::Dir;
use chrono::NaiveDateTime;
use dcb_util::{array_split, ascii_str_arr::AsciiChar, AsciiStrArr};
use std::convert::TryFrom;

/// A directory entry
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum DirEntry {
	/// File
	File {
		/// File name
		name: AsciiStrArr<0x10>,

		/// File extension
		extension: AsciiStrArr<0x3>,

		/// File date
		date: NaiveDateTime,

		/// Contents
		contents: Vec<u8>,
	},

	/// Directory
	Dir {
		/// Directory name
		name: AsciiStrArr<0x10>,

		/// Directory date
		date: NaiveDateTime,

		/// Directory
		dir: Dir,
	},
}

impl DirEntry {
	/// Parses a directory entry from bytes
	pub fn from_bytes(entry_bytes: &[u8; 0x20], file_bytes: &[u8]) -> Result<Self, FromBytesError> {
		let entry_bytes = array_split!(entry_bytes,
			kind      :  0x1,
			extension : [0x3],
			sector_pos: [0x4],
			size      : [0x4],
			data      : [0x4],
			name      : [0x10],
		);

		// Check kind
		let kind = match entry_bytes.kind {
			0x1 => Kind::File,
			0x80 => Kind::Dir,
			&kind => return Err(FromBytesError::InvalidKind(kind)),
		};

		// Special case some files which cause problems and return early, as if we encountered the final entry.
		#[allow(clippy::single_match)] // We'll add more matches in the future
		match entry_bytes.name {
			[0x83, 0x52, 0x83, 0x53, 0x81, 0x5B, 0x20, 0x81, 0x60, 0x20, 0x43, 0x41, 0x52, 0x44, 0x32, 0x00] => {
				return Err(FromBytesError::InvalidKind(0))
			},
			_ => (),
		}

		// Then get the name and contents of this file
		let mut name = AsciiStrArr::from_bytes(entry_bytes.name).map_err(FromBytesError::Name)?;
		name.trim_end(AsciiChar::Null);

		let sector_pos = LittleEndian::read_u32(entry_bytes.sector_pos);
		let size = LittleEndian::read_u32(entry_bytes.size);
		let date = NaiveDateTime::from_timestamp(i64::from(LittleEndian::read_u32(entry_bytes.data)), 0);

		// Get this entry's contents
		let start = 2048 * usize::try_from(sector_pos).expect("Start sector didn't fit into a `usize`");


		match kind {
			Kind::File => {
				let end = start + usize::try_from(size).expect("Start sector didn't fit into a `usize`");
				let contents = file_bytes.get(start..end).ok_or(FromBytesError::ContentsFile(start..end))?;

				let mut extension = AsciiStrArr::from_bytes(entry_bytes.extension).map_err(FromBytesError::Extension)?;
				extension.trim_end(AsciiChar::Null);
				Ok(Self::File {
					name,
					extension,
					date,
					contents: contents.to_vec(),
				})
			},
			Kind::Dir => {
				// Note: No size on directories, so we are unbounded
				let contents = file_bytes.get(start..).ok_or(FromBytesError::ContentsDir(start..))?;
				let dir = Dir::from_bytes(contents, file_bytes).map_err(|err| FromBytesError::ParseDir(Box::new(err)))?;
				Ok(Self::Dir { name, dir, date })
			},
		}
	}
}

/// Enum helper for [`DirEntry::from_bytes`]
enum Kind {
	/// File
	File,

	/// Directory
	Dir,
}
