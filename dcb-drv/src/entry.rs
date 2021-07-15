#![doc = include_str!("entry.md")]

// Modules
mod error;

// Exports
pub use error::DeserializeBytesError;


// Imports
use super::ptr::{DirPtr, FilePtr};
use byteorder::{ByteOrder, LittleEndian};
use chrono::NaiveDateTime;
use dcb_bytes::Bytes;
use std::convert::TryInto;
use zutil::{ascii_str_arr::AsciiChar, AsciiStrArr};

/// A directory entry kind
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum DirEntryKind {
	/// A file
	File {
		/// Extension
		extension: AsciiStrArr<0x3>,

		/// Pointer
		ptr: FilePtr,
	},

	/// Directory
	Dir {
		/// Pointer
		ptr: DirPtr,
	},
}

impl DirEntryKind {
	/// Creates a file kind
	#[must_use]
	pub const fn file(extension: AsciiStrArr<0x3>, ptr: FilePtr) -> Self {
		Self::File { extension, ptr }
	}

	/// Creates a directory kind
	#[must_use]
	pub const fn dir(ptr: DirPtr) -> Self {
		Self::Dir { ptr }
	}

	/// Returns this kind as a file pointer
	#[must_use]
	pub const fn as_file_ptr(&self) -> Option<FilePtr> {
		match *self {
			Self::File { ptr, .. } => Some(ptr),
			_ => None,
		}
	}

	/// Returns this kind as a directory pointer
	#[must_use]
	pub const fn as_dir_ptr(&self) -> Option<DirPtr> {
		match *self {
			Self::Dir { ptr } => Some(ptr),
			_ => None,
		}
	}

	/// Returns the sector position of this entry
	#[must_use]
	pub const fn sector_pos(&self) -> u32 {
		match self {
			DirEntryKind::File { ptr, .. } => ptr.sector_pos,
			DirEntryKind::Dir { ptr } => ptr.sector_pos,
		}
	}
}

/// A directory entry
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct DirEntry {
	/// Entry name
	pub name: AsciiStrArr<0x10>,

	/// Entry date
	pub date: NaiveDateTime,

	/// Entry kind
	pub kind: DirEntryKind,
}

impl Bytes for DirEntry {
	type ByteArray = [u8; 0x20];
	type DeserializeError = DeserializeBytesError;
	type SerializeError = !;

	fn deserialize_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::DeserializeError> {
		let bytes = zutil::array_split!(bytes,
			kind      :  0x1,
			extension : [0x3],
			sector_pos: [0x4],
			size      : [0x4],
			date      : [0x4],
			name      : [0x10],
		);

		// Then get the name and extension
		let mut name = AsciiStrArr::from_bytes(bytes.name).map_err(DeserializeBytesError::Name)?;
		name.trim_end(AsciiChar::Null);
		let mut extension = AsciiStrArr::from_bytes(bytes.extension).map_err(DeserializeBytesError::Extension)?;
		extension.trim_end(AsciiChar::Null);

		// Get the sector position, size and date
		let sector_pos = LittleEndian::read_u32(bytes.sector_pos);
		let size = LittleEndian::read_u32(bytes.size);
		let date = NaiveDateTime::from_timestamp(i64::from(LittleEndian::read_u32(bytes.date)), 0);

		// Check kind
		let kind = match bytes.kind {
			0x1 => DirEntryKind::File {
				extension,
				ptr: FilePtr { sector_pos, size },
			},
			0x80 => {
				debug_assert_eq!(size, 0, "Directory size wasn't 0");
				debug_assert_eq!(extension.len(), 0, "Directory extension wasn't 0");
				DirEntryKind::Dir {
					ptr: DirPtr { sector_pos },
				}
			},
			&kind => return Err(DeserializeBytesError::InvalidKind(kind)),
		};

		Ok(Self { name, date, kind })
	}

	fn serialize_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::SerializeError> {
		let bytes = zutil::array_split_mut!(bytes,
			kind      :  0x1,
			extension : [0x3],
			sector_pos: [0x4],
			size      : [0x4],
			date      : [0x4],
			name      : [0x10],
		);

		// Get the kind, extension and size
		let (kind, extension, sector_pos, size) = match self.kind {
			DirEntryKind::File { extension, ptr } => (0x1, extension, ptr.sector_pos, ptr.size),
			DirEntryKind::Dir { ptr } => (0x80, AsciiStrArr::new(), ptr.sector_pos, 0),
		};

		*bytes.kind = kind;
		let extension = extension.as_bytes();
		bytes.extension[..extension.len()].copy_from_slice(extension);
		bytes.extension[extension.len()..].fill(0);

		LittleEndian::write_u32(bytes.size, size);

		// Then set the name
		let name = self.name.as_bytes();
		bytes.name[..name.len()].copy_from_slice(name);
		bytes.name[name.len()..].fill(0);

		// And the sector
		LittleEndian::write_u32(bytes.sector_pos, sector_pos);

		// Write the date by saturating it if it's too large or small.
		let date = self
			.date
			.timestamp()
			.clamp(0, i64::from(u32::MAX))
			.try_into()
			.expect("Seconds didn't fit into date");
		LittleEndian::write_u32(bytes.date, date);

		Ok(())
	}
}
