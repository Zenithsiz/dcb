//! Directory writer

// Modules
pub mod error;
pub mod lister;

// Exports
pub use error::WriteEntriesError;
pub use lister::DirWriterLister;

// Imports
use super::entry::DirEntryWriterKind;
use crate::{DirEntry, DirEntryKind, DirPtr, FilePtr};
use std::{
	convert::{TryFrom, TryInto},
	io::{self, SeekFrom},
};

/// Directory writer
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct DirWriter<L: DirWriterLister> {
	/// Writer list
	entries: L,
}

impl<L: DirWriterLister> DirWriter<L> {
	/// Creates a new directory writer
	pub fn new(entries: L) -> Self {
		Self { entries }
	}

	/// Writes all entries into a writer
	///
	/// Returns the number of _sectors_ written by all entries
	/// in the directory, excluding the directory itself.
	pub fn write_entries<W: io::Write + io::Seek>(self, writer: &mut W) -> Result<u32, WriteEntriesError<L::Error>> {
		// Get the sector we're currently on
		let start_pos = writer.stream_position().map_err(WriteEntriesError::GetSectorPos)?;
		if start_pos % 2048 != 0 {
			return Err(WriteEntriesError::WriterNotAtSectorStart);
		}
		let start_sector_pos =
			u32::try_from(start_pos / 2048).map_err(|_err| WriteEntriesError::WriterSectorPastMax)?;

		// Get the starting sector position for the first entry.
		// Note: We start right after this directory
		// Note: `+2047` is to pad this directory to the next sector, if not empty.
		let entries: Vec<_> = self
			.entries
			.into_iter()
			.collect::<Result<_, _>>()
			.map_err(WriteEntriesError::GetEntry)?;
		let entries_len: u32 = entries
			.len()
			.try_into()
			.expect("Number of entries didn't fit into a `u32`");
		let mut cur_sector_pos = start_sector_pos + ((entries_len + 1) * 0x20 + 2047) / 2048;

		// Write each entry and map it so we can write it later
		let entries: Vec<_> = entries
			.into_iter()
			.map(|entry: crate::DirEntryWriter<_>| {
				// Create the directory entry to write later
				let dir_entry = DirEntry {
					name: entry.name,
					date: entry.date,
					kind: match &entry.kind {
						DirEntryWriterKind::File(file) => DirEntryKind::File {
							extension: file.extension,
							ptr:       FilePtr::new(cur_sector_pos, file.size()),
						},
						DirEntryWriterKind::Dir(_) => DirEntryKind::Dir {
							ptr: DirPtr::new(cur_sector_pos),
						},
					},
				};

				// Seek to the entry and write it on file
				writer
					.seek(SeekFrom::Start(u64::from(cur_sector_pos) * 2048))
					.map_err(WriteEntriesError::SeekToEntry)?;
				let sector_size = match entry.kind {
					DirEntryWriterKind::File(file) => {
						let size = file.size();
						file.write(writer).map_err(WriteEntriesError::WriteFile)?;
						(size + 2047) / 2048
					},
					DirEntryWriterKind::Dir(dir) => dir
						.write_entries(writer)
						.map_err(|err| WriteEntriesError::WriteDir(Box::new(err)))?,
				};

				// Update our sector pos
				cur_sector_pos += sector_size;

				Ok(dir_entry)
			})
			.collect::<Result<_, _>>()?;

		// Then write the directory
		DirPtr::new(start_sector_pos)
			.write_entries(writer, entries)
			.map_err(WriteEntriesError::WriteEntries)?;

		// And return the number of sectors we wrote.
		Ok(cur_sector_pos - start_sector_pos)
	}
}
