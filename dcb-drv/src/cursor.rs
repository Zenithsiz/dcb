//! Filesystem cursor

// Modules
pub mod error;

// Exports
pub use error::{FindError, NewError, OpenFileError, SwapFilesError};

// Imports
use crate::{DirEntry, DirEntryKind, DirPtr, FilePtr, Path};
use bit_vec::BitVec;
use chrono::NaiveDateTime;
use dcb_util::{AsciiStrArr, IoCursor};
use std::{
	collections::BTreeSet,
	convert::{TryFrom, TryInto},
	io::{self, SeekFrom},
};

/// Filesystem cursor
#[derive(PartialEq, Clone, Debug)]
pub struct DrvFsCursor {
	/// Root directory
	root_dir: DirCursor,

	/// All sectors' status
	sector_status: BitVec,

	/// All files
	files: BTreeSet<FilePtr>,

	/// All directories
	dirs: BTreeSet<DirPtr>,
}

impl DrvFsCursor {
	/// Creates a new filesystem cursor
	pub fn new<T: io::Read + io::Seek>(cursor: &mut T) -> Result<Self, NewError> {
		/// Helper function that sets sector status given a directory
		fn iter_file_tree<R: io::Read + io::Seek>(
			cursor: &mut R, ptr: DirPtr, sector_status: &mut BitVec, files: &mut BTreeSet<FilePtr>,
			dirs: &mut BTreeSet<DirPtr>,
		) -> Result<DirCursor, NewError> {
			// Read all entries
			let entries: Vec<DirEntry> = ptr
				.read_entries(cursor)
				.map_err(|err| NewError::ReadDir {
					sector_pos: ptr.sector_pos,
					err,
				})?
				.collect::<Result<_, _>>()
				.map_err(|err| NewError::ReadDirEntry {
					sector_pos: ptr.sector_pos,
					err,
				})?;

			// Set the entries of the directory as filled
			let dir_sector = usize::try_from(ptr.sector_pos).expect("Sector position didn't fit into `usize`");
			let dir_sectors_len = ((entries.len() + 0x1) * 0x20 + 0x7ff) / 0x800;
			for n in 0..dir_sectors_len {
				sector_status.set(dir_sector + n, true);
			}

			// Then convert all dir entries to our entries
			let entries = entries
				.into_iter()
				.map(|entry| {
					let kind = match entry.kind {
						DirEntryKind::Dir { ptr } => {
							// Add this directory
							dirs.insert(ptr);

							let dir = iter_file_tree(cursor, ptr, sector_status, files, dirs)?;

							DirEntryCursorKind::Dir(dir)
						},
						DirEntryKind::File { extension, ptr } => {
							// Set the file as filled
							let file_sector =
								usize::try_from(ptr.sector_pos).expect("Sector position didn't fit into `usize`");
							let file_sectors_len =
								usize::try_from((ptr.size + 0x7ff) / 0x800).expect("File size didn't fit into `usize`");
							for n in 0..file_sectors_len {
								sector_status.set(file_sector + n, true);
							}

							// Add this file
							// TODO: Maybe allow hard link in the future?
							assert!(files.insert(ptr), "Two files on file lead to the same storage");

							DirEntryCursorKind::File(FileCursor { extension, ptr })
						},
					};

					Ok(DirEntryCursor {
						name: entry.name,
						date: entry.date,
						kind,
					})
				})
				.collect::<Result<_, _>>()?;

			Ok(DirCursor { ptr, entries })
		}

		// Parse the full filesystem, accumulating the status
		let size: usize = cursor
			.stream_len()
			.map_err(NewError::FileSize)?
			.try_into()
			.expect("File size didn't fit into `usize`");
		let mut sector_status = BitVec::from_elem((size + 0x7ff) / 0x800, false);
		let mut files = BTreeSet::new();
		let mut dirs = BTreeSet::new();
		let root_dir = iter_file_tree(cursor, DirPtr::root(), &mut sector_status, &mut files, &mut dirs)?;

		Ok(Self {
			root_dir,
			sector_status,
			files,
			dirs,
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
	pub fn open_file<T: io::Seek + io::Read>(&mut self, cursor: T, path: &Path) -> Result<OpenFile<T>, OpenFileError> {
		let file = match self
			.root_dir_mut()
			.find_mut(path)
			.map(|entry| &mut entry.kind)
			.map_err(OpenFileError::FindFile)?
		{
			DirEntryCursorKind::File(file) => file,
			_ => return Err(OpenFileError::OpenDir),
		};


		Ok(OpenFile {
			inner: file.ptr.cursor(cursor).map_err(OpenFileError::OpenFile)?,
			drive: self,
		})
	}

	/// Swaps two files
	#[allow(clippy::too_many_lines)] // TODO: Refactor
	pub fn swap_files<T: io::Seek + io::Write>(
		&mut self, cursor: &mut T, lhs_path: &Path, rhs_path: &Path,
	) -> Result<(), SwapFilesError> {
		// If both paths are equal, return Error
		if lhs_path == rhs_path {
			return Err(SwapFilesError::BothPathsEqual);
		}

		// Get the common ancestor between both paths
		let (common_path, lhs_path, rhs_path) = lhs_path.common_ancestor(rhs_path);
		let (lhs_common_entry_name, lhs_path) = lhs_path.split_first().ok_or(SwapFilesError::SwapDirs)?;
		let (rhs_common_entry_name, rhs_path) = rhs_path.split_first().ok_or(SwapFilesError::SwapDirs)?;

		// Get the directory at the common point
		let common_dir: &mut DirCursor = self
			.root_dir_mut()
			.find_mut(common_path)
			.map_err(SwapFilesError::FindCommonPath)?
			.kind
			.as_dir_mut()
			.ok_or(SwapFilesError::CommonPathFile)?;

		// Then find each common entry separately
		let mut lhs_common_entry = None;
		let mut rhs_common_entry = None;
		for (idx, entry) in common_dir.entries.iter_mut().enumerate() {
			// If we found them, set them
			match entry.name.as_str() {
				name if name == lhs_common_entry_name.as_str() => lhs_common_entry = Some((idx, entry)),
				name if name == rhs_common_entry_name.as_str() => rhs_common_entry = Some((idx, entry)),
				_ => continue,
			}

			// If we got them both
			if lhs_common_entry.is_some() && rhs_common_entry.is_some() {
				break;
			}
		}

		// Then get the directories
		let (lhs_common_entry_idx, lhs_common_entry) = lhs_common_entry.ok_or(SwapFilesError::CommonPathLhsEntry)?;
		let (rhs_common_entry_idx, rhs_common_entry) = rhs_common_entry.ok_or(SwapFilesError::CommonPathRhsEntry)?;

		// Then find the directory of the files to swap
		let (lhs_dir_ptr, lhs_dir_idx, lhs_file) = match lhs_path.split_last() {
			Some((lhs_dir_path, lhs_filename)) => {
				let lhs_dir_entry = lhs_common_entry
					.kind
					.as_dir_mut()
					.ok_or(SwapFilesError::LhsFileDirEntries)?
					.find_mut(lhs_dir_path)
					.map_err(SwapFilesError::FindLhs)?;
				let lhs_dir = lhs_dir_entry
					.kind
					.as_dir_mut()
					.ok_or(SwapFilesError::LhsFileDirEntries)?;

				let (lhs_dir_idx, lhs_entry) = lhs_dir
					.entries
					.iter_mut()
					.enumerate()
					.find(|(_, entry)| entry.name.as_str() == lhs_filename.as_str())
					.ok_or(SwapFilesError::FindLhsFile)?;

				let lhs_file = lhs_entry.kind.as_file_mut().ok_or(SwapFilesError::SwapDirs)?;

				(lhs_dir.ptr, lhs_dir_idx, lhs_file)
			},
			// If we don't have at least 2 components, the parent is the common dir
			None => (
				common_dir.ptr,
				lhs_common_entry_idx,
				lhs_common_entry.kind.as_file_mut().ok_or(SwapFilesError::SwapDirs)?,
			),
		};
		let (rhs_dir_ptr, rhs_dir_idx, rhs_file) = match rhs_path.split_last() {
			Some((rhs_dir_path, rhs_filename)) => {
				let rhs_dir_entry = rhs_common_entry
					.kind
					.as_dir_mut()
					.ok_or(SwapFilesError::LhsFileDirEntries)?
					.find_mut(rhs_dir_path)
					.map_err(SwapFilesError::FindLhs)?;
				let rhs_dir = rhs_dir_entry
					.kind
					.as_dir_mut()
					.ok_or(SwapFilesError::LhsFileDirEntries)?;

				let (rhs_dir_idx, rhs_entry) = rhs_dir
					.entries
					.iter_mut()
					.enumerate()
					.find(|(_, entry)| entry.name.as_str() == rhs_filename.as_str())
					.ok_or(SwapFilesError::FindLhsFile)?;

				let rhs_file = rhs_entry.kind.as_file_mut().ok_or(SwapFilesError::SwapDirs)?;

				(rhs_dir.ptr, rhs_dir_idx, rhs_file)
			},
			// If we don't have at least 2 components, the parent is the common dir
			None => (
				common_dir.ptr,
				rhs_common_entry_idx,
				rhs_common_entry.kind.as_file_mut().ok_or(SwapFilesError::SwapDirs)?,
			),
		};


		// Swap the paths on disk
		lhs_dir_ptr.seek_to(cursor).map_err(SwapFilesError::SeekLhsEntry)?;
		cursor
			.seek(SeekFrom::Current(
				0x20 * i64::try_from(lhs_dir_idx).expect("Entry number didn't fit into `i64`") + 0x4,
			))
			.map_err(SwapFilesError::SeekLhsEntry)?;
		cursor
			.write(&rhs_file.ptr.sector_pos.to_le_bytes())
			.map_err(SwapFilesError::WriteLhsEntry)?;

		rhs_dir_ptr.seek_to(cursor).map_err(SwapFilesError::SeekRhsEntry)?;
		cursor
			.seek(SeekFrom::Current(
				0x20 * i64::try_from(rhs_dir_idx).expect("Entry number didn't fit into `i64`") + 0x4,
			))
			.map_err(SwapFilesError::SeekLhsEntry)?;
		cursor
			.write(&lhs_file.ptr.sector_pos.to_le_bytes())
			.map_err(SwapFilesError::WriteLhsEntry)?;

		// Then swap the pointers
		std::mem::swap(&mut lhs_file.ptr, &mut rhs_file.ptr);

		Ok(())
	}
}

/// A directory
#[derive(PartialEq, Clone, Debug)]
pub struct DirCursor {
	/// Dir pointer
	ptr: DirPtr,

	/// All entries
	entries: Vec<DirEntryCursor>,
}

impl DirCursor {
	/// Returns all entries
	#[must_use]
	pub fn entries(&self) -> &[DirEntryCursor] {
		&self.entries
	}

	/// Returns all entries mutably
	#[must_use]
	pub fn entries_mut(&mut self) -> &mut [DirEntryCursor] {
		&mut self.entries
	}

	/// Finds a directory entry from within this directory
	pub fn find_mut(&mut self, path: &Path) -> Result<&mut DirEntryCursor, FindError> {
		let mut cur_dir = self;

		let mut components = path.components().peekable();
		while let Some((_, entry_name)) = components.next() {
			let entry = cur_dir
				.entries
				.iter_mut()
				.find(|entry| entry.name.as_str() == entry_name.as_str())
				.ok_or(FindError::FindFile)?;

			// If we don't have anything next, return the entry
			if components.peek().is_none() {
				return Ok(entry);
			}

			match &mut entry.kind {
				// If we got a file return error
				DirEntryCursorKind::File(_) => return Err(FindError::FileDirEntries),

				// If we got a directory, continue
				DirEntryCursorKind::Dir(dir) => cur_dir = dir,
			};
		}

		// If we get here, the path was empty
		Err(FindError::EmptyPath)
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

impl DirEntryCursorKind {
	/// Returns this entry kind as a directory
	pub fn as_dir_mut(&mut self) -> Option<&mut DirCursor> {
		match self {
			Self::Dir(v) => Some(v),
			_ => None,
		}
	}

	/// Returns this entry kind as a file
	pub fn as_file_mut(&mut self) -> Option<&mut FileCursor> {
		match self {
			Self::File(v) => Some(v),
			_ => None,
		}
	}
}

/// A file cursor
#[derive(PartialEq, Clone, Debug)]
pub struct FileCursor {
	/// Extension
	extension: AsciiStrArr<0x3>,

	/// File pointer
	ptr: FilePtr,
}

impl FileCursor {
	/// Get a reference to the file cursor's extension.
	#[must_use]
	pub const fn extension(&self) -> &AsciiStrArr<0x3> {
		&self.extension
	}

	/// Returns the pointer of this file cursor
	#[must_use]
	pub const fn ptr(&self) -> FilePtr {
		self.ptr
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
