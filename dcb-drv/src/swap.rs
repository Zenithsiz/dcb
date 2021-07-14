//! File swapping

// Imports
use crate::{ptr, DirEntryKind, DirPtr, Path};
use std::{io, mem};

/// Swaps two files
pub fn swap_files<T: io::Seek + io::Read + io::Write>(
	cursor: &mut T, lhs_path: &Path, rhs_path: &Path,
) -> Result<(), SwapFilesError> {
	// Find both files and their entry pointers
	let (lhs_entry_ptr, mut lhs_entry) = DirPtr::root().find(cursor, lhs_path).map_err(SwapFilesError::FindLhs)?;
	let (rhs_entry_ptr, mut rhs_entry) = DirPtr::root().find(cursor, rhs_path).map_err(SwapFilesError::FindLhs)?;

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
	/// Unable to find lhs file
	#[error("Unable to find lhs file")]
	FindLhs(#[source] ptr::dir::FindError),

	/// Unable to find rhs file
	#[error("Unable to find rhs file")]
	FindRhs(#[source] ptr::dir::FindError),

	/// Both paths must be files
	#[error("Both paths must be files")]
	BothPathsMustBeFiles,

	/// Unable to write lhs file entry
	#[error("Unable to write lhs file entry")]
	WriteLhs(#[source] ptr::entry::WriteEntryError),

	/// Unable to write rhs file entry
	#[error("Unable to write rhs file entry")]
	WriteRhs(#[source] ptr::entry::WriteEntryError),
}
