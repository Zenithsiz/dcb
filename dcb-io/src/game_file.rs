//! Game file.
//!
//! See [`GameFile`] for details

// Modules
mod error;
pub mod path;

// Exports
pub use error::{OpenFileError, SwapFilesError};
pub use path::Path;

// Imports
use dcb_drv::DirEntryKind;
use std::io;
use zutil::IoSlice;

/// Game file.
#[derive(PartialEq, Clone, Debug)]
pub struct GameFile<T> {
	/// CD-Rom
	cdrom: T,
}

// Constants
impl<T> GameFile<T> {
	/// `A.DRV` Offset
	pub const A_OFFSET: u64 = 0xa1000;
	/// `A.DRV` Size
	pub const A_SIZE: u64 = 0xa78800;
	/// `B.DRV` Offset
	pub const B_OFFSET: u64 = 0xb19800;
	/// `B.DRV` Size
	pub const B_SIZE: u64 = 0x17ea800;
	/// `C.DRV` Offset
	pub const C_OFFSET: u64 = 0x2304000;
	/// `C.DRV` Size
	pub const C_SIZE: u64 = 0xb6c000;
	/// `E.DRV` Offset
	pub const E_OFFSET: u64 = 0x2e70000;
	/// `E.DRV` Size
	pub const E_SIZE: u64 = 0x1886800;
	/// `F.DRV` Offset
	pub const F_OFFSET: u64 = 0x46f6800;
	/// `F.DRV` Size
	pub const F_SIZE: u64 = 0xf2f800;
	/// `G.DRV` Offset
	pub const G_OFFSET: u64 = 0x5626000;
	/// `G.DRV` Size
	pub const G_SIZE: u64 = 0x293000;
	/// `P.DRV` Offset
	pub const P_OFFSET: u64 = 0xc000;
	/// `P.DRV` Size
	pub const P_SIZE: u64 = 0x95000;
}

// Constructors
impl<T: io::Read + io::Seek> GameFile<T> {
	/// Creates a new game file
	pub fn new(cdrom: T) -> Self {
		Self { cdrom }
	}
}

// Getters
impl<T> GameFile<T> {
	/// Returns the cdrom associated with this game file
	pub fn cdrom(&mut self) -> &mut T {
		&mut self.cdrom
	}
}

// Drive getters
impl<T: io::Seek> GameFile<T> {
	/// Returns the `A.DRV` file alongside it's cursor
	pub fn a_drv(&mut self) -> Result<DriveCursor<&mut T>, io::Error> {
		match DriveCursor::new(&mut self.cdrom, Self::A_OFFSET, Self::A_SIZE) {
			Ok(cursor) => Ok(cursor),
			Err(err) => Err(err),
		}
	}

	/// Returns the `B.DRV` file alongside it's cursor
	pub fn b_drv(&mut self) -> Result<DriveCursor<&mut T>, io::Error> {
		match DriveCursor::new(&mut self.cdrom, Self::B_OFFSET, Self::B_SIZE) {
			Ok(cursor) => Ok(cursor),
			Err(err) => Err(err),
		}
	}

	/// Returns the `C.DRV` file alongside it's cursor
	pub fn c_drv(&mut self) -> Result<DriveCursor<&mut T>, io::Error> {
		match DriveCursor::new(&mut self.cdrom, Self::C_OFFSET, Self::C_SIZE) {
			Ok(cursor) => Ok(cursor),
			Err(err) => Err(err),
		}
	}

	/// Returns the `E.DRV` file alongside it's cursor
	pub fn e_drv(&mut self) -> Result<DriveCursor<&mut T>, io::Error> {
		match DriveCursor::new(&mut self.cdrom, Self::E_OFFSET, Self::E_SIZE) {
			Ok(cursor) => Ok(cursor),
			Err(err) => Err(err),
		}
	}

	/// Returns the `F.DRV` file alongside it's cursor
	pub fn f_drv(&mut self) -> Result<DriveCursor<&mut T>, io::Error> {
		match DriveCursor::new(&mut self.cdrom, Self::F_OFFSET, Self::F_SIZE) {
			Ok(cursor) => Ok(cursor),
			Err(err) => Err(err),
		}
	}

	/// Returns the `G.DRV` file alongside it's cursor
	pub fn g_drv(&mut self) -> Result<DriveCursor<&mut T>, io::Error> {
		match DriveCursor::new(&mut self.cdrom, Self::G_OFFSET, Self::G_SIZE) {
			Ok(cursor) => Ok(cursor),
			Err(err) => Err(err),
		}
	}

	/// Returns the `P.DRV` file alongside it's cursor
	pub fn p_drv(&mut self) -> Result<DriveCursor<&mut T>, io::Error> {
		match DriveCursor::new(&mut self.cdrom, Self::P_OFFSET, Self::P_SIZE) {
			Ok(cursor) => Ok(cursor),
			Err(err) => Err(err),
		}
	}
}

// Files
impl<T: io::Seek + io::Read> GameFile<T> {
	/// Opens a file
	pub fn open_file(&mut self, path: &Path) -> Result<FileCursor<DriveCursor<&mut T>>, OpenFileError> {
		// Check the drive we're accessing.
		let (drive, path) = path.drive().ok_or(OpenFileError::NoDrive)?;
		let mut cursor = match drive.as_char() {
			'A' => self.a_drv().map_err(OpenFileError::OpenDrive)?,
			'B' => self.b_drv().map_err(OpenFileError::OpenDrive)?,
			'C' => self.c_drv().map_err(OpenFileError::OpenDrive)?,
			'E' => self.e_drv().map_err(OpenFileError::OpenDrive)?,
			'F' => self.f_drv().map_err(OpenFileError::OpenDrive)?,
			'G' => self.g_drv().map_err(OpenFileError::OpenDrive)?,
			'P' => self.p_drv().map_err(OpenFileError::OpenDrive)?,
			drive => return Err(OpenFileError::UnknownDrive { drive }),
		};

		// Then get the entry
		let (_, entry) = dcb_drv::DirPtr::root()
			.find(&mut cursor, path)
			.map_err(OpenFileError::FindFile)?;

		match entry.kind {
			DirEntryKind::File { ptr, .. } => ptr.cursor(cursor).map_err(OpenFileError::OpenFile),
			_ => Err(OpenFileError::FoundDir),
		}
	}
}

impl<T: io::Seek + io::Read + io::Write> GameFile<T> {
	/// Swaps two files
	pub fn swap_files(&mut self, lhs: &Path, rhs: &Path) -> Result<(), SwapFilesError> {
		// Check the drive we're accessing.
		let (lhs_drive, lhs_path) = lhs.drive().ok_or(SwapFilesError::NoDrive)?;
		let (rhs_drive, rhs_path) = rhs.drive().ok_or(SwapFilesError::NoDrive)?;
		let mut cursor = match (lhs_drive.as_char(), rhs_drive.as_char()) {
			('A', 'A') => self.a_drv().map_err(SwapFilesError::OpenDrive)?,
			('B', 'B') => self.b_drv().map_err(SwapFilesError::OpenDrive)?,
			('C', 'C') => self.c_drv().map_err(SwapFilesError::OpenDrive)?,
			('E', 'E') => self.e_drv().map_err(SwapFilesError::OpenDrive)?,
			('F', 'F') => self.f_drv().map_err(SwapFilesError::OpenDrive)?,
			('G', 'G') => self.g_drv().map_err(SwapFilesError::OpenDrive)?,
			('P', 'P') => self.p_drv().map_err(SwapFilesError::OpenDrive)?,
			(drive, _) if lhs_drive == rhs_drive => return Err(SwapFilesError::UnknownDrive { drive }),
			_ => return Err(SwapFilesError::AcrossDrives),
		};

		// Then swap both files
		dcb_drv::swap_files(&mut cursor, lhs_path, rhs_path).map_err(SwapFilesError::SwapFiles)
	}
}

/// Driver cursor
pub type DriveCursor<T> = IoSlice<T>;

/// File cursor
// TODO: Make proper file cursor in `dcb-drv` that allows expanding
pub type FileCursor<T> = IoSlice<T>;
