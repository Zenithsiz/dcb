//! Directory entry

// Modules
pub mod error;

// Exports
pub use error::FromBytesError;

// Imports
use byteorder::{ByteOrder, LittleEndian};
use chrono::NaiveDateTime;
use dcb_bytes::Bytes;
use dcb_util::{array_split, array_split_mut, ascii_str_arr::AsciiChar, AsciiStrArr};
use std::{
	convert::TryFrom,
	io::{self, Seek, SeekFrom},
};

/// A directory entry kind
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum DirEntryKind {
	/// A file
	File {
		/// File extension
		extension: AsciiStrArr<0x3>,

		/// Size
		size: u32,
	},

	/// Directory
	Dir,
}

/// A directory entry
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct DirEntry {
	/// Entry name
	pub name: AsciiStrArr<0x10>,

	/// Entry date
	pub date: NaiveDateTime,

	/// Sector position
	pub sector_pos: u32,

	/// Entry kind
	pub kind: DirEntryKind,
}

impl DirEntry {
	/// Seeks to this entry's data on a reader
	pub fn seek_to<R: Seek>(&self, reader: &mut R) -> Result<u64, io::Error> {
		reader.seek(SeekFrom::Start(u64::from(self.sector_pos) * 2048))
	}
}

impl Bytes for DirEntry {
	type ByteArray = [u8; 0x20];
	type FromError = FromBytesError;
	type ToError = !;

	fn from_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::FromError> {
		let bytes = array_split!(bytes,
			kind      :  0x1,
			extension : [0x3],
			sector_pos: [0x4],
			size      : [0x4],
			data      : [0x4],
			name      : [0x10],
		);

		// Check kind
		let kind = match bytes.kind {
			0x1 => {
				let mut extension = AsciiStrArr::from_bytes(bytes.extension).map_err(FromBytesError::Extension)?;
				extension.trim_end(AsciiChar::Null);
				let size = LittleEndian::read_u32(bytes.size);

				DirEntryKind::File { extension, size }
			},
			0x80 => DirEntryKind::Dir,
			&kind => return Err(FromBytesError::InvalidKind(kind)),
		};

		// Special case some files which cause problems and return early, as if we encountered the final entry.
		// TODO: Generalize this somehow
		#[allow(clippy::single_match)] // We'll add more matches in the future
		match bytes.name {
			[0x83, 0x52, 0x83, 0x53, 0x81, 0x5B, 0x20, 0x81, 0x60, 0x20, 0x43, 0x41, 0x52, 0x44, 0x32, 0x00] => {
				return Err(FromBytesError::InvalidKind(0))
			},
			_ => (),
		}

		// Then get the name and other common metadata
		let mut name = AsciiStrArr::from_bytes(bytes.name).map_err(FromBytesError::Name)?;
		name.trim_end(AsciiChar::Null);
		let sector_pos = LittleEndian::read_u32(bytes.sector_pos);
		let date = NaiveDateTime::from_timestamp(i64::from(LittleEndian::read_u32(bytes.data)), 0);

		Ok(Self {
			name,
			date,
			sector_pos,
			kind,
		})
	}

	fn to_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::ToError> {
		let bytes = array_split_mut!(bytes,
			kind      :  0x1,
			extension : [0x3],
			sector_pos: [0x4],
			size      : [0x4],
			data      : [0x4],
			name      : [0x10],
		);

		match &self.kind {
			DirEntryKind::File { extension, size } => {
				*bytes.kind = 0x1;
				let extension = extension.as_bytes();
				bytes.extension[..extension.len()].copy_from_slice(extension);
				bytes.extension[extension.len()..].fill(0);
				LittleEndian::write_u32(bytes.size, *size);
			},
			DirEntryKind::Dir => {
				*bytes.kind = 0x80;
			},
		};

		// Then set the name and other common metadata
		let name = self.name.as_bytes();
		bytes.name[..name.len()].copy_from_slice(name);
		bytes.name[name.len()..].fill(0);

		// TODO: Support dates after by either returning error or saturating.
		LittleEndian::write_u32(bytes.sector_pos, self.sector_pos);
		let secs = self.date.timestamp();
		let secs = match u32::try_from(secs) {
			Ok(secs) => secs,
			Err(_) => match secs {
				secs if secs < 0 => 0,
				secs if secs > i64::from(u32::MAX) => u32::MAX,
				_ => unreachable!(),
			},
		};
		LittleEndian::write_u32(bytes.data, secs);

		Ok(())
	}
}
