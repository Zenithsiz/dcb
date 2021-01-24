//! Directory entry

// Modules
pub mod error;

// Exports
pub use error::FromBytesError;

use crate::drv::FileReader;

// Imports
use super::DirReader;
use byteorder::{ByteOrder, LittleEndian};
use chrono::NaiveDateTime;
use dcb_bytes::Bytes;
use dcb_util::{array_split, array_split_mut, ascii_str_arr::AsciiChar, AsciiStrArr};
use std::convert::TryFrom;

/// A directory entry kind
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum DirEntryKind {
	/// A file
	File(FileReader),

	/// Directory
	Dir(DirReader),
}

/// A directory entry
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct DirEntry {
	/// Entry name
	name: AsciiStrArr<0x10>,

	/// Entry date
	date: NaiveDateTime,

	/// Entry kind
	kind: DirEntryKind,
}

impl DirEntry {
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
	pub const fn kind(&self) -> &DirEntryKind {
		&self.kind
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

		let sector_pos = LittleEndian::read_u32(bytes.sector_pos);

		// Check kind
		let kind = match bytes.kind {
			0x1 => {
				let mut extension = AsciiStrArr::from_bytes(bytes.extension).map_err(FromBytesError::Extension)?;
				extension.trim_end(AsciiChar::Null);
				let size = LittleEndian::read_u32(bytes.size);

				DirEntryKind::File(FileReader::new(extension, sector_pos, size))
			},
			0x80 => DirEntryKind::Dir(DirReader::new(sector_pos)),
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
		let date = NaiveDateTime::from_timestamp(i64::from(LittleEndian::read_u32(bytes.data)), 0);

		Ok(Self { name, date, kind })
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
			DirEntryKind::File(file) => {
				*bytes.kind = 0x1;

				let extension = file.extension().as_bytes();
				bytes.extension[..extension.len()].copy_from_slice(extension);
				bytes.extension[extension.len()..].fill(0);

				LittleEndian::write_u32(bytes.sector_pos, file.sector_pos());
				LittleEndian::write_u32(bytes.size, file.size());
			},
			DirEntryKind::Dir(dir) => {
				*bytes.kind = 0x80;

				LittleEndian::write_u32(bytes.sector_pos, dir.sector_pos());
			},
		};

		// Then set the name and other common metadata
		let name = self.name.as_bytes();
		bytes.name[..name.len()].copy_from_slice(name);
		bytes.name[name.len()..].fill(0);

		// Write the date by saturating it if it's too large or small.
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
