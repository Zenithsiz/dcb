//! Errors

// Imports
use std::io;

/// Error for [`GameFile::new`](super::GameFile::new)
#[derive(Debug, thiserror::Error)]
pub enum NewError {
	/// Unable to open `A.DRV`
	#[error("Unable to open `A.DRV` file")]
	OpenA(#[source] io::Error),

	/// Unable to create `A.DRV` cursor
	#[error("Unable to create `A.DRV` cursor")]
	CursorA(#[source] dcb_drv::cursor::NewError),

	/// Unable to open `B.DRV`
	#[error("Unable to open `B.DRV` file")]
	OpenB(#[source] io::Error),

	/// Unable to create `B.DRV` cursor
	#[error("Unable to create `B.DRV` cursor")]
	CursorB(#[source] dcb_drv::cursor::NewError),

	/// Unable to open `C.DRV`
	#[error("Unable to open `C.DRV` file")]
	OpenC(#[source] io::Error),

	/// Unable to create `C.DRV` cursor
	#[error("Unable to create `C.DRV` cursor")]
	CursorC(#[source] dcb_drv::cursor::NewError),

	/// Unable to open `E.DRV`
	#[error("Unable to open `E.DRV` file")]
	OpenE(#[source] io::Error),

	/// Unable to create `E.DRV` cursor
	#[error("Unable to create `E.DRV` cursor")]
	CursorE(#[source] dcb_drv::cursor::NewError),

	/// Unable to open `F.DRV`
	#[error("Unable to open `F.DRV` file")]
	OpenF(#[source] io::Error),

	/// Unable to create `F.DRV` cursor
	#[error("Unable to create `F.DRV` cursor")]
	CursorF(#[source] dcb_drv::cursor::NewError),

	/// Unable to open `G.DRV`
	#[error("Unable to open `G.DRV` file")]
	OpenG(#[source] io::Error),

	/// Unable to create `G.DRV` cursor
	#[error("Unable to create `G.DRV` cursor")]
	CursorG(#[source] dcb_drv::cursor::NewError),

	/// Unable to open `P.DRV`
	#[error("Unable to open `P.DRV` file")]
	OpenP(#[source] io::Error),

	/// Unable to create `P.DRV` cursor
	#[error("Unable to create `P.DRV` cursor")]
	CursorP(#[source] dcb_drv::cursor::NewError),
}

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

	/// Unable to open file
	#[error("Unable to open file")]
	OpenFile(#[source] dcb_drv::cursor::OpenFileError),
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
	SwapFiles(#[source] dcb_drv::cursor::SwapFilesError),
}
