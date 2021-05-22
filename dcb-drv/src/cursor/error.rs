//! Errors

use crate::dir::reader::{ReadEntriesError, ReadEntryError};
use std::io;

/// Error for [`Dir::read_entries`](super::Dir::read_entries)
#[derive(Debug, thiserror::Error)]
pub enum NewError {
	/// Unable to get file size
	#[error("Unable to get file size")]
	FileSize(#[source] io::Error),

	/// Unable to read directory
	#[error("Unable to read directory at {sector_pos:#x}")]
	ReadDir {
		/// Position of the sector
		sector_pos: u32,

		/// Underlying error
		#[source]
		err: ReadEntriesError,
	},

	/// Unable to read directory entry
	#[error("Unable to read directory entry at {sector_pos:#x}")]
	ReadDirEntry {
		/// Position of the sector
		sector_pos: u32,

		/// Underlying error
		#[source]
		err: ReadEntryError,
	},
}
