//! Entry finding

// Imports
use crate::{DirEntry, DirEntryKind, DirEntryPtr, DirPtr, Path, PathBuf};
use std::io;

/// Finds an entry given it's path
pub fn find_entry<R: io::Seek + io::Read>(
	reader: &mut R, path: &Path,
) -> Result<(DirEntryPtr, DirEntry), FindEntryError> {
	let mut cur_dir_ptr = DirPtr::root();

	let mut components = path.components();
	while let Some(entry_name) = components.next() {
		// Find the entry
		let (entry_ptr, entry) = cur_dir_ptr
			.find_entry(reader, entry_name.as_ascii())
			.map_err(FindEntryError::FindEntry)?;

		// If there's no component after this, return the current entry
		if components.clone().next().is_none() {
			return Ok((entry_ptr, entry));
		}

		// Else check what entry we got
		match entry.kind {
			// If we got a file at this stage return error
			DirEntryKind::File { .. } => {
				return Err(FindEntryError::ExpectedDir {
					path: path[..(path.len() - components.remaining().len())].to_path_buf(),
				})
			},

			// If we got a directory, continue
			DirEntryKind::Dir { ptr } => cur_dir_ptr = ptr,
		};
	}

	// If we get here, the path was empty
	Err(FindEntryError::EmptyPath)
}

/// Error type for [`find_entry`]
#[derive(Debug, thiserror::Error)]
pub enum FindEntryError {
	/// Unable to find entry
	#[error("Unable to find entry")]
	FindEntry(#[source] crate::ptr::FindEntryError),

	/// Expected directory
	#[error("Expected directory at {path}")]
	ExpectedDir {
		/// The path that wasn't a directory
		path: PathBuf,
	},

	/// Path was empty
	#[error("Path was empty")]
	EmptyPath,
}
