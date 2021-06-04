//! Errors

// Imports
use dcb_drv::ptr;
use std::io;


/// Error for [`GameFile::open_file`](super::GameFile::open_file)
#[derive(Debug, thiserror::Error)]
pub enum OpenFileError {
	/// No drive specified
	#[error("No drive specified")]
	NoDrive,

	/// Unknown drive specified
	#[error("Unknown drive {drive} specified")]
	UnknownDrive {
		/// Drive found
		drive: char,
	},

	/// Unable to open drive
	#[error("Unable to open drive")]
	OpenDrive(#[source] io::Error),

	/// Unable to find file
	#[error("Unable to find file")]
	FindFile(#[source] ptr::FindError),

	/// Found directory
	#[error("Found directory")]
	FoundDir,

	/// Unable to open file
	#[error("Unable to open file")]
	OpenFile(#[source] ptr::FileCursorError),
}


/// Error for [`GameFile::swap_files`](super::GameFile::swap_files)
#[derive(Debug, thiserror::Error)]
pub enum SwapFilesError {
	/// No drive specified
	#[error("No drive specified")]
	NoDrive,

	/// Unknown drive specified
	#[error("Unknown drive {drive} specified")]
	UnknownDrive {
		/// Drive found
		drive: char,
	},

	/// Swapping can only be done across the same drive currently
	#[error("Swapping can only be done across the same drive currently")]
	AcrossDrives,

	/// Unable to open drive
	#[error("Unable to open drive")]
	OpenDrive(#[source] io::Error),

	/// Unable to swap files
	#[error("Unable to swap files")]
	SwapFiles(#[source] dcb_drv::swap::SwapFilesError),
}
