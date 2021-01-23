//! Abstraction over the game file.
//!
//! See [`GameFile`] for details

// Modules
pub mod error;

use dcb_iso9660::entry::FileReader;
// Exports
pub use error::{NewError, ReadFileError};

// Imports
use dcb_cdrom_xa::CdRom;
use std::io;

/// Game file reader.
#[derive(PartialEq, Eq, Debug)]
pub struct GameFile<'a, R> {
	/// CD-Rom
	cdrom: &'a mut CdRom<R>,

	/// Iso9660 filesystem
	filesystem: dcb_iso9660::Filesystem,
}

// Constructors
impl<'a, R: io::Read + io::Seek> GameFile<'a, R> {
	/// Creates a new game file from the cd reader
	pub fn new(cdrom: &'a mut CdRom<R>) -> Result<Self, NewError> {
		// Read the filesystem
		let filesystem = dcb_iso9660::Filesystem::new(cdrom).map_err(NewError::ParseFilesystem)?;

		Ok(Self { cdrom, filesystem })
	}
}

impl<'a, R> GameFile<'a, R> {
	/// Returns the cdrom associated with this game file
	pub fn cdrom(&mut self) -> &mut CdRom<R> {
		self.cdrom
	}
}


impl<'a, R: io::Read + io::Seek> GameFile<'a, R> {
	/// Reads a game file
	pub fn read_drv<'b>(&'b mut self, name: &str) -> Result<FileReader<'b, R>, ReadFileError>
	where
		'a: 'b,
	{
		// Read the root directory
		let root_dir = self.filesystem.root_dir().read_dir(self.cdrom).map_err(ReadFileError::ReadRoot)?;

		// Get the file
		let entry = root_dir.find(name).ok_or(ReadFileError::FindFile)?;

		// And read it
		entry.read_file(self.cdrom).map_err(ReadFileError::ReadFile)
	}
}
