//! Filesystem cursor

// Modules
pub mod error;

// Exports
pub use error::{NewError, OpenFileError};

// Imports
use crate::{dir::entry::DirEntryReaderKind, DirEntryReader, DirReader};
use bit_vec::BitVec;
use chrono::NaiveDateTime;
use dcb_util::{AsciiStrArr, IoCursor};
use std::{
	convert::{TryFrom, TryInto},
	io,
};

/// Filesystem cursor
#[derive(PartialEq, Clone, Debug)]
pub struct DrvFsCursor {
	/// Root directory
	root_dir: DirCursor,

	/// All sectors' status
	sector_status: BitVec,
}

impl DrvFsCursor {
	/// Creates a new filesystem cursor
	pub fn new<R: io::Read + io::Seek>(reader: &mut R) -> Result<Self, NewError> {
		/// Helper function that sets sector status given a directory
		fn iter_file_tree<R: io::Read + io::Seek>(
			cursor: &mut R, dir: DirReader, sector_status: &mut BitVec,
		) -> Result<DirCursor, NewError> {
			// Read all entries
			// TODO: Avoid allocations by going through all entries twice.
			let entries_reader: Vec<DirEntryReader> = dir
				.read_entries(cursor)
				.map_err(|err| NewError::ReadDir {
					sector_pos: dir.sector_pos(),
					err,
				})?
				.collect::<Result<_, _>>()
				.map_err(|err| NewError::ReadDirEntry {
					sector_pos: dir.sector_pos(),
					err,
				})?;

			// Set the entries of the directory as filled
			let dir_sector = usize::try_from(dir.sector_pos()).expect("Sector position didn't fit into `usize`");
			let dir_sectors_len = ((entries_reader.len() + 0x1) * 0x20 + 0x7ff) / 0x800;
			for n in 0..dir_sectors_len {
				sector_status.set(dir_sector + n, true);
			}

			// Then add all entries
			let mut entries = vec![];
			for entry in entries_reader {
				let kind = match *entry.kind() {
					DirEntryReaderKind::Dir(dir) => {
						let dir = iter_file_tree(cursor, dir, sector_status)?;

						DirEntryCursorKind::Dir(dir)
					},
					DirEntryReaderKind::File(file) => {
						// Set the file as filled
						let file_sector =
							usize::try_from(file.sector_pos()).expect("Sector position didn't fit into `usize`");
						let file_sectors_len =
							usize::try_from((file.size() + 0x7ff) / 0x800).expect("File size didn't fit into `usize`");
						for n in 0..file_sectors_len {
							sector_status.set(file_sector + n, true);
						}

						DirEntryCursorKind::File(FileCursor {
							extension:  *file.extension(),
							sector_pos: file.sector_pos(),
							size:       file.size(),
						})
					},
				};

				let entry = DirEntryCursor {
					name: *entry.name(),
					date: entry.date(),
					kind,
				};
				entries.push(entry);
			}

			Ok(DirCursor {
				sector_pos: dir.sector_pos(),
				entries,
			})
		}

		// Go through all directories and files and create a status
		let size: usize = reader
			.stream_len()
			.map_err(NewError::FileSize)?
			.try_into()
			.expect("File size didn't fit into `usize`");
		let mut sector_status = BitVec::from_elem((size + 0x7ff) / 0x800, false);

		let root_dir = iter_file_tree(reader, DirReader::new(0), &mut sector_status)?;

		Ok(Self {
			root_dir,
			sector_status,
		})
	}

	/// Returns the root directory
	#[must_use]
	pub const fn root_dir(&self) -> &DirCursor {
		&self.root_dir
	}

	/// Returns the root directory mutably
	#[must_use]
	pub fn root_dir_mut(&mut self) -> &mut DirCursor {
		&mut self.root_dir
	}

	/// Opens a file
	pub fn open_file<T: io::Seek + io::Read>(
		&mut self, cursor: T, mut path: &str,
	) -> Result<OpenFile<T>, OpenFileError> {
		let mut cur_dir = self.root_dir();
		loop {
			// Check if we need to go any more directories in
			match path.split_once('\\') {
				// If so, find the directory in the current directory
				Some((dir, new_path)) => {
					path = new_path;

					cur_dir = match cur_dir
						.entries
						.iter()
						.find(|entry| entry.name.as_str() == dir)
						.map(|entry| &entry.kind)
					{
						Some(DirEntryCursorKind::Dir(dir)) => dir,
						Some(_) => return Err(OpenFileError::FileDirEntries),
						None => return Err(OpenFileError::FindFile),
					};
				},

				// If not, open the file in the current directory
				None => {
					return match cur_dir
						.entries
						.iter()
						.find(|entry| entry.name.as_str() == path)
						.map(|entry| &entry.kind)
					{
						Some(DirEntryCursorKind::File(file)) => Ok(OpenFile {
							inner: IoCursor::new(cursor, u64::from(file.sector_pos) * 0x800, u64::from(file.size))
								.map_err(OpenFileError::OpenFile)?,
							drive: self,
						}),
						Some(_) => Err(OpenFileError::OpenDir),
						None => Err(OpenFileError::FindFile),
					}
				},
			}
		}

		//
	}
}

/// A directory
#[derive(PartialEq, Clone, Debug)]
pub struct DirCursor {
	/// Sector position
	sector_pos: u32,

	/// All entries
	entries: Vec<DirEntryCursor>,
}

impl DirCursor {
	/// Returns all entries
	#[must_use]
	pub fn entries(&self) -> &[DirEntryCursor] {
		&self.entries
	}
}

/// A directory entry cursor
#[derive(PartialEq, Clone, Debug)]
pub struct DirEntryCursor {
	/// Entry name
	name: AsciiStrArr<0x10>,

	/// Entry date
	date: NaiveDateTime,

	/// Kind
	kind: DirEntryCursorKind,
}

impl DirEntryCursor {
	/// Get a reference to the dir entry cursor's name.
	#[must_use]
	pub const fn name(&self) -> &AsciiStrArr<0x10> {
		&self.name
	}

	/// Get a reference to the dir entry cursor's date.
	#[must_use]
	pub const fn date(&self) -> &NaiveDateTime {
		&self.date
	}

	/// Get a reference to the dir entry cursor's kind.
	#[must_use]
	pub const fn kind(&self) -> &DirEntryCursorKind {
		&self.kind
	}
}

/// A directory entry kind
#[derive(PartialEq, Clone, Debug)]
pub enum DirEntryCursorKind {
	/// Directory
	Dir(DirCursor),

	/// File
	File(FileCursor),
}

/// A file cursor
#[derive(PartialEq, Clone, Debug)]
pub struct FileCursor {
	/// Extension
	extension: AsciiStrArr<0x3>,

	/// Sector position
	sector_pos: u32,

	/// Size
	size: u32,
}

impl FileCursor {
	/// Get a reference to the file cursor's extension.
	#[must_use]
	pub const fn extension(&self) -> &AsciiStrArr<0x3> {
		&self.extension
	}

	/// Get a reference to the file cursor's sector pos.
	#[must_use]
	pub const fn sector_pos(&self) -> &u32 {
		&self.sector_pos
	}

	/// Get a reference to the file cursor's size.
	#[must_use]
	pub const fn size(&self) -> &u32 {
		&self.size
	}
}


/// An opened file
#[derive(Debug)]
pub struct OpenFile<'a, T> {
	/// Drive
	drive: &'a mut DrvFsCursor,

	/// Inner
	inner: IoCursor<T>,
}

impl<'a, T: io::Seek + io::Read> io::Read for OpenFile<'a, T> {
	fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
		self.inner.read(buf)
	}
}

impl<'a, T: io::Seek> io::Seek for OpenFile<'a, T> {
	fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
		// TODO: Allow file to expand beyond here too.
		self.inner.seek(pos)
	}
}

impl<'a, T: io::Seek + io::Write> io::Write for OpenFile<'a, T> {
	fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
		// TODO: Allow file to expand beyond
		self.inner.write(buf)
	}

	fn flush(&mut self) -> io::Result<()> {
		self.inner.flush()
	}
}
