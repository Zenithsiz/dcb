//! Pointers

// Modules
mod error;

// Exports
pub use error::{
	FileCursorError, FindEntryError, FindError, ReadEntriesError, ReadEntryError, WriteEntriesError, WriteEntryError,
};

// Imports
use super::DirEntry;
use crate::{path, DirEntryKind, Path};
use ascii::AsciiStr;
use core::str::lossy::Utf8Lossy;
use dcb_bytes::Bytes;
use std::io::{self, SeekFrom};
use zutil::IoSlice;

/// File pointer
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct FilePtr {
	/// Sector position
	pub sector_pos: u32,

	/// Size
	pub size: u32,
}

impl FilePtr {
	/// Creates a new file pointer
	#[must_use]
	pub const fn new(sector_pos: u32, size: u32) -> Self {
		Self { sector_pos, size }
	}

	/// Seeks to this directory on a cursor
	pub fn seek_to<T: io::Seek>(self, cursor: &mut T) -> Result<u64, io::Error> {
		cursor.seek(SeekFrom::Start(u64::from(self.sector_pos) * 0x800))
	}

	/// Returns a cursor for this file
	pub fn cursor<T: io::Seek>(self, cursor: T) -> Result<IoSlice<T>, FileCursorError> {
		let pos = u64::from(self.sector_pos) * 0x800;
		IoSlice::new(cursor, pos, u64::from(self.size)).map_err(FileCursorError::Seek)
	}
}

impl PartialOrd for FilePtr {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for FilePtr {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		// Only compare sector position
		self.sector_pos.cmp(&other.sector_pos)
	}
}

/// Directory pointer
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct DirPtr {
	/// Sector position
	pub sector_pos: u32,
}

impl DirPtr {
	/// Creates a new directory pointer
	#[must_use]
	pub const fn new(sector_pos: u32) -> Self {
		Self { sector_pos }
	}

	/// Returns the root directory pointer
	#[must_use]
	pub const fn root() -> Self {
		Self { sector_pos: 0 }
	}

	/// Seeks to this directory on a cursor
	pub fn seek_to<T: io::Seek>(self, cursor: &mut T) -> Result<u64, io::Error> {
		cursor.seek(SeekFrom::Start(u64::from(self.sector_pos) * 0x800))
	}

	/// Returns an iterator over all entries in this directory
	pub fn read_entries<R: io::Read + io::Seek>(
		self, reader: &mut R,
	) -> Result<impl Iterator<Item = Result<DirEntry, ReadEntryError>> + '_, ReadEntriesError> {
		// Seek to the sector
		self.seek_to(reader).map_err(ReadEntriesError::Seek)?;

		// Then create the iterator
		let iter = std::iter::from_fn(move || {
			let entry: Result<_, _> = try {
				// Read the bytes
				let mut bytes = [0; 0x20];
				reader.read_exact(&mut bytes).map_err(ReadEntryError::ReadEntry)?;

				// If all bytes are null, we're finished
				if bytes == [0; 0x20] {
					return None;
				}

				// Special case some entries which cause problems
				#[allow(clippy::single_match)] // We might add more matches in the future
				match &bytes {
					b"\x01CDD\xd5/\x00\x00\xf0?\x01\x00\xe6u\xad:\x83R\x83S\x81[ \x81` CARD2\x00" => {
						log::warn!("Ignoring special directory entry: {:?}", Utf8Lossy::from_bytes(&bytes));
						return None;
					},
					_ => (),
				}

				// Else parse it
				Some(DirEntry::deserialize_bytes(&bytes).map_err(ReadEntryError::ParseEntry)?)
			};

			entry.transpose()
		});

		Ok(iter)
	}

	/// Finds an entry from it's path
	pub fn find<R: io::Seek + io::Read>(
		self, reader: &mut R, path: &Path,
	) -> Result<(DirEntryPtr, DirEntry), FindError> {
		// Current directory pointer
		let mut cur_ptr = self;

		// Current entry
		let mut cur_entry = None;

		let mut components = path.components();
		loop {
			match components.next() {
				// If we get root, reset us to root and clear any entry we have
				Some(path::Component::Root) => (cur_ptr, cur_entry) = (DirPtr::root(), None),

				// For current directory just get the next component
				Some(path::Component::CurDir) => continue,

				// Return `Err` on parent directories
				// Note: We don't support parent directories as we'd have to store all
				//       of the parent directories, because directories don't have
				//       access to their parents
				// TODO: Using recursion / stack allocation outside of the loop we could easily store
				//       all parent dirs we've been through, without any heap allocations.
				Some(path::Component::ParentDir) => return Err(FindError::ParentDir),

				// On a normal entry, find the entry in the current dir
				Some(path::Component::Normal(entry_name)) => {
					// Find the entry
					let (entry_ptr, entry) = cur_ptr.find_entry(reader, entry_name).map_err(FindError::FindEntry)?;

					// If this is the final entry, return it
					if components.clone().next().is_none() {
						return Ok((entry_ptr, entry));
					}

					// Else check what entry we got
					match entry.kind {
						DirEntryKind::File { .. } => {
							return Err(FindError::ExpectedDir {
								path: path[..(path.len() - components.remaining().len())].to_path_buf(),
							})
						},

						// If we got a directory, continue
						DirEntryKind::Dir { ptr } => {
							cur_entry = Some((entry_ptr, entry));
							cur_ptr = ptr;
						},
					};
				},

				// If we're done, return whatever entry we had before running out
				None => return cur_entry.ok_or(FindError::EmptyPath),
			}
		}
	}

	/// Finds an entry
	pub fn find_entry<R: io::Read + io::Seek>(
		self, reader: &mut R, entry_name: &AsciiStr,
	) -> Result<(DirEntryPtr, DirEntry), FindEntryError> {
		let (filename, extension) = entry_name
			.as_str()
			.split_once('.')
			.map_or((entry_name.as_str(), None), |(filename, extension)| {
				(filename, Some(extension))
			});

		self.read_entries(reader)
			.map_err(FindEntryError::SeekDir)?
			.zip(0..)
			.find_map(|(entry, idx)| match entry {
				Ok(entry) => {
					let is_match = entry.name.as_str() == filename &&
						match entry.kind {
							DirEntryKind::Dir { .. } => extension.is_none(),
							DirEntryKind::File { extension: ext, .. } => extension == Some(ext.as_str()),
						};

					match is_match {
						true => Some(Ok((DirEntryPtr::new(self, idx), entry))),
						false => None,
					}
				},
				Err(err) => Some(Err(err)),
			})
			.ok_or(FindEntryError::FindEntry)?
			.map_err(FindEntryError::ReadEntry)
	}

	/// Writes a list of entries to a writer
	pub fn write_entries<W: io::Seek + io::Write>(
		self, writer: &mut W, entries: impl IntoIterator<Item = DirEntry>,
	) -> Result<(), WriteEntriesError> {
		// Seek to the sector
		self.seek_to(writer).map_err(WriteEntriesError::Seek)?;

		// For each entry, write it
		for entry in entries {
			// Put the entry into bytes
			let mut entry_bytes = [0; 0x20];
			entry.serialize_bytes(&mut entry_bytes).into_ok();

			// Then write it
			writer.write_all(&entry_bytes).map_err(WriteEntriesError::WriteEntry)?;
		}

		Ok(())
	}
}

/// Directory entry pointer
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct DirEntryPtr {
	/// Directory
	dir: DirPtr,

	/// Entry
	entry: u32,
}

impl DirEntryPtr {
	/// Creates a new entry pointer
	#[must_use]
	pub const fn new(dir: DirPtr, entry: u32) -> Self {
		Self { dir, entry }
	}

	/// Seeks to this entry on a cursor
	pub fn seek_to<T: io::Seek>(self, cursor: &mut T) -> Result<u64, io::Error> {
		cursor.seek(SeekFrom::Start(
			u64::from(self.dir.sector_pos) * 0x800 + u64::from(self.entry) * 0x20,
		))
	}

	/// Writes an entry to this pointer
	pub fn write<W: io::Seek + io::Write>(self, writer: &mut W, entry: &DirEntry) -> Result<(), WriteEntryError> {
		// Seek to this entry
		self.seek_to(writer).map_err(WriteEntryError::Seek)?;

		// Then write the entry
		let mut entry_bytes = [0; 0x20];
		entry.serialize_bytes(&mut entry_bytes).into_ok();

		// Then write it
		writer.write_all(&entry_bytes).map_err(WriteEntryError::WriteEntry)
	}
}
