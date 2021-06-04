//! File swapping

// Imports
use crate::{
	find::FindEntryError,
	ptr::{self, WriteEntryError},
	DirEntryKind, DirPtr, Path,
};
use std::{io, mem};

/// Swaps two files within a file
pub fn swap_files<T: io::Seek + io::Read + io::Write>(
	cursor: &mut T, lhs_path: &Path, rhs_path: &Path,
) -> Result<(), SwapFilesError> {
	// Read both directories
	let (lhs_dir_ptr, lhs_filename) = match lhs_path.split_last() {
		Some((lhs_dir_path, lhs_filename)) => {
			let lhs_dir_ptr = crate::find_entry(cursor, lhs_dir_path)
				.map_err(SwapFilesError::LhsDir)?
				.kind
				.as_dir_ptr()
				.ok_or(SwapFilesError::LhsParentIsFile)?;
			(lhs_dir_ptr, lhs_filename)
		},
		_ => (DirPtr::root(), lhs_path.as_ascii()),
	};
	let (rhs_dir_ptr, rhs_filename) = match rhs_path.split_last() {
		Some((rhs_dir_path, rhs_filename)) => {
			let rhs_dir_ptr = crate::find_entry(cursor, rhs_dir_path)
				.map_err(SwapFilesError::RhsDir)?
				.kind
				.as_dir_ptr()
				.ok_or(SwapFilesError::RhsParentIsFile)?;
			(rhs_dir_ptr, rhs_filename)
		},
		_ => (DirPtr::root(), rhs_path.as_ascii()),
	};

	// Read the directory entries and find where the file is
	let (lhs_entry_ptr, mut lhs_entry) = lhs_dir_ptr
		.find_entry(cursor, lhs_filename)
		.map_err(SwapFilesError::FindLhsFile)?;
	let (rhs_entry_ptr, mut rhs_entry) = rhs_dir_ptr
		.find_entry(cursor, rhs_filename)
		.map_err(SwapFilesError::FindRhsFile)?;

	// Swap both entries' file pointers
	match (&mut lhs_entry.kind, &mut rhs_entry.kind) {
		(DirEntryKind::File { ptr: lhs_ptr, .. }, DirEntryKind::File { ptr: rhs_ptr, .. }) => {
			mem::swap(lhs_ptr, rhs_ptr);
		},
		_ => return Err(SwapFilesError::BothPathsMustBeFiles),
	}

	// Then write both entries back
	lhs_entry_ptr
		.write(cursor, &lhs_entry)
		.map_err(SwapFilesError::WriteLhs)?;
	rhs_entry_ptr
		.write(cursor, &rhs_entry)
		.map_err(SwapFilesError::WriteRhs)?;

	Ok(())
}

/// Error type for [`swap_files`]
#[derive(Debug, thiserror::Error)]
pub enum SwapFilesError {
	/// Unable to get lhs directory
	#[error("Unable to get lhs directory")]
	LhsDir(#[source] FindEntryError),

	/// Unable to get rhs directory
	#[error("Unable to get rhs directory")]
	RhsDir(#[source] FindEntryError),

	/// Lhs parent was a file
	#[error("Lhs parent was a file")]
	LhsParentIsFile,

	/// Rhs parent was a file
	#[error("Rhs parent was a file")]
	RhsParentIsFile,

	/// Unable to find lhs filename
	#[error("Unable to find lhs filename")]
	FindLhsFile(#[source] ptr::FindEntryError),

	/// Unable to find lhs filename
	#[error("Unable to find rhs filename")]
	FindRhsFile(#[source] ptr::FindEntryError),

	/// Both paths must be files
	#[error("Both paths must be files")]
	BothPathsMustBeFiles,

	/// Unable to write lhs file entry
	#[error("Unable to write lhs file entry")]
	WriteLhs(#[source] WriteEntryError),

	/// Unable to write rhs file entry
	#[error("Unable to write rhs file entry")]
	WriteRhs(#[source] WriteEntryError),
}
