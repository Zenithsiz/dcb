//! Writer

// Imports
use crate::{DirEntry, DirEntryKind, DirPtr, FilePtr};
use chrono::NaiveDateTime;
use zutil::{AsciiStrArr, MapBoxResult};
use std::{
	convert::TryInto,
	io::{self, SeekFrom},
};

/// A directory lister
pub trait DirWriterLister:
	Sized + IntoIterator<Item = Result<DirEntryWriter<Self>, Self::Error>, IntoIter: ExactSizeIterator>
{
	/// File type
	type FileReader: io::Read;

	/// Error type for each entry
	type Error: std::error::Error + 'static;
}

/// Error for [`DirWriter::write`]
#[derive(Debug, thiserror::Error)]
pub enum WriteDirError<E: std::error::Error + 'static> {
	/// Unable to get entry
	#[error("Unable to get entry")]
	GetEntry(#[source] E),

	/// Unable to seek to entry
	#[error("Unable to seek to entry")]
	SeekToEntry(#[source] io::Error),

	/// Unable to write file
	#[error("Unable to write file")]
	WriteFile(#[source] io::Error),

	/// File size was too large
	#[error("File size was too large")]
	FileTooLarge,

	/// Unable to write directory
	#[error("Unable to write directory")]
	WriteDir(#[source] Box<Self>),

	/// Unable to write all directory entries
	#[error("Unable to write directory entries")]
	WriteEntries(#[source] crate::ptr::WriteEntriesError),
}

/// A directory writer
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct DirWriter<L> {
	/// All entries
	entries: L,
}

impl<L> DirWriter<L> {
	/// Creates a new directory writer
	#[must_use]
	pub const fn new(entries: L) -> Self {
		Self { entries }
	}
}

impl<L: DirWriterLister> DirWriter<L> {
	/// Writes `entries` to this directory recursively and returns the number of
	/// sectors occupied
	pub fn write<W: io::Seek + io::Write>(self, ptr: DirPtr, writer: &mut W) -> Result<u32, WriteDirError<L::Error>> {
		// Get the starting sector position for the first entry.
		// Note: We on the directory after this directory.
		// Note: `+1` for the null entry.
		// Note: `+2047` is to pad this directory to the next sector, if not empty.
		let entries = self.entries.into_iter();
		let entries_len: u32 = entries
			.len()
			.try_into()
			.expect("Number of entries didn't fit into a `u32`");
		let mut cur_sector_pos = ptr.sector_pos + ((entries_len + 1) * 0x20 + 2047) / 2048;

		// Write each entry and map it so we can write it later
		let entries: Vec<_> = entries
			.map(|entry| {
				// Get the entry
				let entry = entry.map_err(WriteDirError::GetEntry)?;

				// Seek to the entry
				writer
					.seek(SeekFrom::Start(u64::from(cur_sector_pos) * 2048))
					.map_err(WriteDirError::SeekToEntry)?;

				// Then write it and get it's sector size
				let (dir_entry_kind, sector_size) = match entry.kind {
					DirEntryWriterKind::File { extension, mut reader } => {
						// Write the file and get the size as `u32`
						let size: u32 = io::copy(&mut reader, writer)
							.map_err(WriteDirError::WriteFile)?
							.try_into()
							.map_err(|_| WriteDirError::FileTooLarge)?;
						let sector_size = (size + 2047) / 2048;

						let ptr = FilePtr::new(cur_sector_pos, sector_size);
						(DirEntryKind::file(extension, ptr), sector_size)
					},
					DirEntryWriterKind::Dir(dir) => {
						// Write all entries recursively
						let ptr = DirPtr::new(cur_sector_pos);
						let sector_size = dir.write(ptr, writer).box_map_err(WriteDirError::WriteDir)?;
						(DirEntryKind::dir(ptr), sector_size)
					},
				};

				// Update our sector pos
				cur_sector_pos += sector_size;

				Ok(DirEntry {
					name: entry.name,
					date: entry.date,
					kind: dir_entry_kind,
				})
			})
			.collect::<Result<_, _>>()?;

		// Then write the entries
		ptr.write_entries(writer, entries)
			.map_err(WriteDirError::WriteEntries)?;

		// And return the number of sectors we wrote.
		Ok(cur_sector_pos - ptr.sector_pos)
	}
}

/// A directory entry writer
pub struct DirEntryWriter<L: DirWriterLister> {
	/// Entry name
	pub name: AsciiStrArr<0x10>,

	/// Entry date
	pub date: NaiveDateTime,

	/// Kind
	pub kind: DirEntryWriterKind<L>,
}

/// A directory entry writer kind
pub enum DirEntryWriterKind<L: DirWriterLister> {
	/// A file
	File {
		/// Extension
		extension: AsciiStrArr<0x3>,

		/// File reader
		reader: L::FileReader,
	},

	/// A directory
	Dir(DirWriter<L>),
}
