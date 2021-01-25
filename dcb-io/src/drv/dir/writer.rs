//! Directory writer

// Modules
pub mod error;

// Exports
pub use error::WriteEntriesError;

// Imports
use super::{entry::DirEntryWriterKind, DirEntryWriter};
use std::{
	convert::TryFrom,
	io::{self, SeekFrom},
};

/// Directory lister
pub trait DirWriterLister: Sized + std::fmt::Debug
where
	Self: IntoIterator<Item = Result<DirEntryWriter<Self>, <Self as DirWriterLister>::Error>>,
{
	/// Reader used for the files in this directory
	type FileReader: std::fmt::Debug + io::Read;

	/// Directory lister for each directory in this directory
	type DirList: DirWriterLister;

	/// Error type for each entry
	type Error: std::error::Error + 'static;

	/// Returns the number of entries in this lister
	fn entries_len(&self) -> u32;
}

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

	/// Returns the number of entries
	pub fn entries_len(&self) -> u32 {
		self.entries.entries_len()
	}

	/// Returns this directory's size
	///
	/// This only returns the size of the directory itself, not of
	/// the sum of it's entries.
	pub fn size(&self) -> u32 {
		// Note: `+1` for the terminator
		(self.entries_len() + 1) * 0x20
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
		let start_sector_pos = u32::try_from(start_pos / 2048).map_err(|_err| WriteEntriesError::WriterSectorPastMax)?;

		// Get the starting sector position for the first entry.
		// Note: We start right after this directory
		// Note: `+2047` is to pad this directory to the next sector, if not empty.
		let mut cur_sector_pos = start_sector_pos + (self.size() + 2047) / 2048;

		// All entries' bytes
		let mut entries_bytes = vec![];

		// For each entry, write it and add it to our directory bytes
		for entry in self.entries {
			// Get the entry
			let entry = entry.map_err(WriteEntriesError::GetEntry)?;

			// Write the entry on our directory
			let mut entry_bytes = [0; 0x20];
			entry.to_bytes(&mut entry_bytes, cur_sector_pos);
			entries_bytes.extend_from_slice(&entry_bytes);

			// Seek to the entry
			writer
				.seek(SeekFrom::Start(u64::from(cur_sector_pos) * 2048))
				.map_err(WriteEntriesError::SeekToEntry)?;

			// Write the entry on the file
			let sector_size = match entry.into_kind() {
				DirEntryWriterKind::File(file) => {
					let size = file.size();
					file.write(writer).map_err(WriteEntriesError::WriteFile)?;
					(size + 2047) / 2048
				},
				DirEntryWriterKind::Dir(dir) => dir.write_entries(writer).map_err(|err| WriteEntriesError::WriteDir(Box::new(err)))?,
			};

			// Update our sector pos
			cur_sector_pos += sector_size;
		}

		// Then seek back to our directory and write it
		writer
			.seek(SeekFrom::Start(u64::from(start_sector_pos) * 2048))
			.map_err(WriteEntriesError::SeekEntries)?;
		writer.write_all(&entries_bytes).map_err(WriteEntriesError::WriteEntries)?;

		// And return the number of sectors we wrote.
		Ok(cur_sector_pos - start_sector_pos)
	}
}
