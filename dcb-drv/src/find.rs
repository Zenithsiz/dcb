//! Entry finding

// Imports
use crate::{path::Component, DirEntry, DirEntryKind, DirEntryPtr, DirPtr, Path, PathBuf};
use std::io;

/// Finds an entry given it's path
pub fn find_entry<R: io::Seek + io::Read>(
	reader: &mut R, path: &Path,
) -> Result<(DirEntryPtr, DirEntry), FindEntryError> {
	// Current directory pointer
	let mut cur_dir_ptr = DirPtr::root();

	// Current entry
	let mut cur_entry = None;

	let mut components = path.components();
	while let Some(component) = components.next() {
		// Get the next entry's name
		let entry_name = match component {
			// Note: We start at the root, so `Root` doesn't do anything to us
			Component::Root | Component::CurDir => continue,
			Component::ParentDir => return Err(FindEntryError::ParentDir),
			Component::Normal(entry) => entry,
		};

		// Find the entry
		let (entry_ptr, entry) = cur_dir_ptr
			.find_entry(reader, entry_name)
			.map_err(FindEntryError::FindEntry)?;

		// If there's no component after this, break
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
			DirEntryKind::Dir { ptr } => {
				cur_entry = Some((entry_ptr, entry));
				cur_dir_ptr = ptr;
			},
		};
	}

	// If we got here, try to return our current entry
	cur_entry.ok_or(FindEntryError::EmptyPath)
}

/// Error type for [`find_entry`]
#[derive(Debug, thiserror::Error)]
pub enum FindEntryError {
	/// Unable to find entry
	#[error("Unable to find entry")]
	FindEntry(#[source] crate::ptr::FindEntryError),

	/// Cannot go back to parent directory
	#[error("Cannot go back to parent directory")]
	ParentDir,

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
