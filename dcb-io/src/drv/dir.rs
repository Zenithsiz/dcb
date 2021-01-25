#![doc(include = "dir.md")]

// Modules
pub mod entry;
pub mod error;

// Exports
pub use entry::{DirEntryReader, DirEntryWriter};
pub use error::{EntriesError, ReadEntryError, WriteEntriesError};

// Imports
use self::entry::DirEntryWriterKind;
use std::{
	convert::TryFrom,
	io::{self, SeekFrom},
};

/// Directory reader
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct DirReader {
	/// Sector position
	sector_pos: u32,
}

impl DirReader {
	/// Creates a directory reader from it's sector
	#[must_use]
	pub const fn new(sector_pos: u32) -> Self {
		Self { sector_pos }
	}

	/// Returns this directory's sector position
	#[must_use]
	pub const fn sector_pos(self) -> u32 {
		self.sector_pos
	}

	/// Seeks to this directory on a reader
	pub fn seek_to<R: io::Seek>(self, reader: &mut R) -> Result<u64, io::Error> {
		reader.seek(SeekFrom::Start(u64::from(self.sector_pos) * 2048))
	}

	/// Returns an iterator over all entries in this directory
	pub fn read_entries<R: io::Read + io::Seek>(
		self, reader: &mut R,
	) -> Result<impl Iterator<Item = Result<DirEntryReader, ReadEntryError>> + '_, EntriesError> {
		// Seek to the sector
		reader
			.seek(SeekFrom::Start(u64::from(self.sector_pos) * 2048))
			.map_err(EntriesError::Seek)?;

		// Then create the iterator
		let iter = std::iter::from_fn(move || {
			// Read the bytes
			let mut entry_bytes = [0; 0x20];
			if let Err(err) = reader.read_exact(&mut entry_bytes) {
				return Some(Err(ReadEntryError::ReadEntry(err)));
			}

			// And parse it
			DirEntryReader::from_bytes(&entry_bytes).map_err(ReadEntryError::ParseEntry).transpose()
		});

		Ok(iter)
	}
}

/// Directory list
pub trait DirWriterList: Sized + std::fmt::Debug {
	/// Reader used for the files in this directory
	type FileReader: std::fmt::Debug + io::Read;

	/// Directory lister
	type DirList: DirWriterList;

	/// Error type for each entry
	type Error: std::error::Error + 'static;

	/// Iterator
	type Iter: Iterator<Item = Result<DirEntryWriter<Self>, Self::Error>>;

	/// Converts this list into an iterator
	fn into_iter(self) -> Self::Iter;
}

/// Directory writer
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct DirWriter<L: DirWriterList> {
	/// Writer list
	entries: L,

	/// Number of entries
	entries_len: u32,
}

impl<L: DirWriterList> DirWriter<L> {
	/// Creates a new directory writer.
	pub fn new(entries: L, entries_len: u32) -> Self {
		Self { entries, entries_len }
	}

	/// Returns the number of entries
	pub fn entries_len(&self) -> u32 {
		self.entries_len
	}

	/// Returns this directory's size
	pub fn size(&self) -> u32 {
		// Note: `+1` for the terminator
		(self.entries_len() + 1) * 0x20
	}

	/// Writes all entries into a writer
	///
	/// Returns the number of sectors written by this directory
	pub fn write_entries<W: io::Write + io::Seek>(self, writer: &mut W) -> Result<u32, WriteEntriesError<L::Error>> {
		// Get the sector we're currently on
		let start_pos = writer.stream_position().map_err(WriteEntriesError::GetPos)?;
		if start_pos % 2048 != 0 {
			return Err(WriteEntriesError::WriterAtSectorStart);
		}
		let start_sector_pos = u32::try_from(start_pos / 2048).map_err(|_err| WriteEntriesError::WriterSectorPastMax)?;

		// Get the starting sector position for the first entry.
		// Note: We start right after this directory
		// Note: `+2047` is to pad this directory to the next sector, if not empty.
		let mut cur_sector_pos = start_sector_pos + (self.size() + 2047) / 2048;

		// Our directory to write after writing all entries
		let mut dir_bytes = vec![];

		// For each entry, write it and add it to our directory bytes
		for entry in self.entries.into_iter() {
			// Get the entry
			let entry = entry.map_err(WriteEntriesError::GetEntry)?;

			// Write the entry on our directory
			let mut entry_bytes = [0; 0x20];
			entry.to_bytes(&mut entry_bytes, cur_sector_pos);
			dir_bytes.extend_from_slice(&entry_bytes);

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

		// Then write our directory
		writer
			.seek(SeekFrom::Start(u64::from(start_sector_pos) * 2048))
			.map_err(WriteEntriesError::SeekToEntries)?;

		writer.write_all(&dir_bytes).map_err(WriteEntriesError::WriteEntries)?;

		Ok(cur_sector_pos - start_sector_pos)
	}
}
