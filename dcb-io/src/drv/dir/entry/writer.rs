//! Directory entry writer

// Imports
use crate::drv::{DirWriter, DirWriterList, FileWriter};
use byteorder::{ByteOrder, LittleEndian};
use chrono::NaiveDateTime;
use dcb_util::{array_split_mut, AsciiStrArr};
use std::convert::TryFrom;

/// A directory entry kind
#[derive(Debug)]
pub enum DirEntryWriterKind<L: DirWriterList> {
	/// A file
	File(FileWriter<L::FileReader>),

	/// Directory
	Dir(DirWriter<L>),
}

/// A directory entry reader
#[derive(Debug)]
pub struct DirEntryWriter<L: DirWriterList> {
	/// Entry name
	name: AsciiStrArr<0x10>,

	/// Entry date
	date: NaiveDateTime,

	/// Entry kind
	kind: DirEntryWriterKind<L>,
}

impl<L: DirWriterList> DirEntryWriter<L> {
	/// Creates a new entry writer from it's name, date and kind
	pub fn new(name: AsciiStrArr<0x10>, date: NaiveDateTime, kind: DirEntryWriterKind<L>) -> Self {
		Self { name, date, kind }
	}

	/// Returns this entry's size
	pub fn size(&self) -> u32 {
		match &self.kind {
			DirEntryWriterKind::File(file) => file.size(),
			DirEntryWriterKind::Dir(dir) => dir.size(),
		}
	}

	/// Returns this entry's kind
	pub fn kind(&self) -> &DirEntryWriterKind<L> {
		&self.kind
	}

	/// Returns this entry's kind
	pub fn into_kind(self) -> DirEntryWriterKind<L> {
		self.kind
	}

	/// Writes this entry to bytes
	pub fn to_bytes(&self, bytes: &mut [u8; 0x20], sector_pos: u32) {
		let bytes = array_split_mut!(bytes,
			kind      :  0x1,
			extension : [0x3],
			sector_pos: [0x4],
			size      : [0x4],
			data      : [0x4],
			name      : [0x10],
		);

		match &self.kind {
			DirEntryWriterKind::File(file) => {
				*bytes.kind = 0x1;

				let extension = file.extension().as_bytes();
				bytes.extension[..extension.len()].copy_from_slice(extension);
				bytes.extension[extension.len()..].fill(0);

				LittleEndian::write_u32(bytes.size, file.size());
			},
			DirEntryWriterKind::Dir(_) => {
				*bytes.kind = 0x80;

				LittleEndian::write_u32(bytes.size, 0);
			},
		};

		// Then set the name
		let name = self.name.as_bytes();
		bytes.name[..name.len()].copy_from_slice(name);
		bytes.name[name.len()..].fill(0);

		// And the sector
		LittleEndian::write_u32(bytes.sector_pos, sector_pos);

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
	}
}
