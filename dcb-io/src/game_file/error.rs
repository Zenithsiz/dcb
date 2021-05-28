//! Errors

// Imports
use std::io;

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
