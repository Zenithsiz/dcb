//! Directory

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
			match DirEntryReader::from_bytes(&entry_bytes) {
				Err(entry::FromBytesError::InvalidKind(0)) => None,
				res => Some(res.map_err(ReadEntryError::ParseEntry)),
			}
		});

		Ok(iter)
	}
}

/// Directory writer
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct DirWriter<R: io::Read, I: ExactSizeIterator<Item = Result<DirEntryWriter<R, I>, io::Error>>> {
	/// Iterator over all entries
	entries: I,
}

impl<R: io::Read, I: ExactSizeIterator<Item = Result<DirEntryWriter<R, I>, io::Error>>> DirWriter<R, I> {
	/// Creates a new directory writer.
	pub fn new(entries: I) -> Self {
		Self { entries }
	}

	/// Returns the number of entries
	pub fn entries_len(&self) -> u32 {
		u32::try_from(self.entries.len()).expect("Too many entries")
	}

	/// Writes all entries into a writer
	pub fn write_entries<W: io::Write + io::Seek>(self, writer: &mut W) -> Result<(), WriteEntriesError> {
		// Get the sector we're currently on
		let sector_pos = writer.stream_position().map_err(WriteEntriesError::GetPos)? / 2048;
		let sector_pos = u32::try_from(sector_pos).expect("`.DRV` file is too big");

		// Get the starting sector pos for each entry
		// Note: We start right after this directory
		let start_sector_pos = sector_pos + (self.entries_len() * 0x20 + 2047) / 2048;

		// Get all the entries with their sector positions
		let entries = self
			.entries
			.scan(start_sector_pos, |cur_sector_pos, res| match res {
				Ok(entry) => {
					let sector_pos = *cur_sector_pos;
					*cur_sector_pos += (entry.size() + 2047) / 2048;
					Some(Ok((entry, sector_pos)))
				},
				Err(err) => Some(Err(err)),
			})
			.collect::<Result<Vec<_>, _>>()
			.map_err(WriteEntriesError::GetEntry)?;

		// Write each entry in the directory
		for (entry, sector_pos) in &entries {
			// Write the bytes
			let mut entry_bytes = [0; 0x20];
			entry.to_bytes(&mut entry_bytes, *sector_pos);

			// And write them
			writer.write_all(&entry_bytes).map_err(WriteEntriesError::WriteEntryInDir)?;
		}

		// Then write each entry
		for (entry, sector_pos) in entries {
			// Seek to the entry
			writer
				.seek(SeekFrom::Start(u64::from(sector_pos) * 2048))
				.map_err(WriteEntriesError::SeekToEntry)?;

			// Write the entry
			match entry.into_kind() {
				DirEntryWriterKind::File(file) => file.into_writer(writer).map_err(WriteEntriesError::WriteFile)?,
				DirEntryWriterKind::Dir(dir) => dir.write_entries(writer).map_err(|err| WriteEntriesError::WriteDir(Box::new(err)))?,
			}
		}

		Ok(())
	}
}
