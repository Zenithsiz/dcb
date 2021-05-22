//! Game file.
//!
//! See [`GameFile`] for details

// Modules
pub mod error;
pub mod path;

// Exports
pub use error::{NewError, OpenFileError};
pub use path::Path;

// Imports
use dcb_cdrom_xa::CdRomCursor;
use dcb_drv::cursor::{DrvFsCursor, OpenFile};
use dcb_util::IoCursor;
use std::io;

/// Game file.
#[derive(PartialEq, Clone, Debug)]
pub struct GameFile<T> {
	/// CD-Rom
	cdrom: CdRomCursor<T>,

	/// `A.DRV` cursor
	a_drv_cursor: DrvFsCursor,

	/// `B.DRV` cursor
	b_drv_cursor: DrvFsCursor,

	/// `C.DRV` cursor
	c_drv_cursor: DrvFsCursor,

	/// `E.DRV` cursor
	e_drv_cursor: DrvFsCursor,

	/// `F.DRV` cursor
	f_drv_cursor: DrvFsCursor,

	/// `G.DRV` cursor
	g_drv_cursor: DrvFsCursor,

	/// `P.DRV` cursor
	p_drv_cursor: DrvFsCursor,
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
	pub fn new(mut cdrom: CdRomCursor<T>) -> Result<Self, NewError> {
		let mut a_drv = IoCursor::new(&mut cdrom, Self::A_OFFSET, Self::A_SIZE).map_err(NewError::OpenA)?;
		let a_drv_cursor = DrvFsCursor::new(&mut a_drv).map_err(NewError::CursorA)?;

		let mut b_drv = IoCursor::new(&mut cdrom, Self::B_OFFSET, Self::B_SIZE).map_err(NewError::OpenB)?;
		let b_drv_cursor = DrvFsCursor::new(&mut b_drv).map_err(NewError::CursorB)?;

		let mut c_drv = IoCursor::new(&mut cdrom, Self::C_OFFSET, Self::C_SIZE).map_err(NewError::OpenC)?;
		let c_drv_cursor = DrvFsCursor::new(&mut c_drv).map_err(NewError::CursorC)?;

		let mut e_drv = IoCursor::new(&mut cdrom, Self::E_OFFSET, Self::E_SIZE).map_err(NewError::OpenE)?;
		let e_drv_cursor = DrvFsCursor::new(&mut e_drv).map_err(NewError::CursorE)?;

		let mut f_drv = IoCursor::new(&mut cdrom, Self::F_OFFSET, Self::F_SIZE).map_err(NewError::OpenF)?;
		let f_drv_cursor = DrvFsCursor::new(&mut f_drv).map_err(NewError::CursorF)?;

		let mut g_drv = IoCursor::new(&mut cdrom, Self::G_OFFSET, Self::G_SIZE).map_err(NewError::OpenG)?;
		let g_drv_cursor = DrvFsCursor::new(&mut g_drv).map_err(NewError::CursorG)?;

		let mut p_drv = IoCursor::new(&mut cdrom, Self::P_OFFSET, Self::P_SIZE).map_err(NewError::OpenP)?;
		let p_drv_cursor = DrvFsCursor::new(&mut p_drv).map_err(NewError::CursorP)?;

		Ok(Self {
			cdrom,
			a_drv_cursor,
			b_drv_cursor,
			c_drv_cursor,
			e_drv_cursor,
			f_drv_cursor,
			g_drv_cursor,
			p_drv_cursor,
		})
	}
}

// Getters
impl<T> GameFile<T> {
	/// Returns the cdrom associated with this game file
	pub fn cdrom(&mut self) -> &mut CdRomCursor<T> {
		&mut self.cdrom
	}
}

// Drive getters
impl<T: io::Seek> GameFile<T> {
	/// Returns the `A.DRV` file alongside it's cursor
	pub fn a_drv(&mut self) -> Result<(&mut DrvFsCursor, DriveCursor<&mut CdRomCursor<T>>), io::Error> {
		match DriveCursor::new(&mut self.cdrom, Self::A_OFFSET, Self::A_SIZE) {
			Ok(cursor) => Ok((&mut self.a_drv_cursor, cursor)),
			Err(err) => Err(err),
		}
	}

	/// Returns the `B.DRV` file alongside it's cursor
	pub fn b_drv(&mut self) -> Result<(&mut DrvFsCursor, DriveCursor<&mut CdRomCursor<T>>), io::Error> {
		match DriveCursor::new(&mut self.cdrom, Self::B_OFFSET, Self::B_SIZE) {
			Ok(cursor) => Ok((&mut self.b_drv_cursor, cursor)),
			Err(err) => Err(err),
		}
	}

	/// Returns the `C.DRV` file alongside it's cursor
	pub fn c_drv(&mut self) -> Result<(&mut DrvFsCursor, DriveCursor<&mut CdRomCursor<T>>), io::Error> {
		match DriveCursor::new(&mut self.cdrom, Self::C_OFFSET, Self::C_SIZE) {
			Ok(cursor) => Ok((&mut self.c_drv_cursor, cursor)),
			Err(err) => Err(err),
		}
	}

	/// Returns the `E.DRV` file alongside it's cursor
	pub fn e_drv(&mut self) -> Result<(&mut DrvFsCursor, DriveCursor<&mut CdRomCursor<T>>), io::Error> {
		match DriveCursor::new(&mut self.cdrom, Self::E_OFFSET, Self::E_SIZE) {
			Ok(cursor) => Ok((&mut self.e_drv_cursor, cursor)),
			Err(err) => Err(err),
		}
	}

	/// Returns the `F.DRV` file alongside it's cursor
	pub fn f_drv(&mut self) -> Result<(&mut DrvFsCursor, DriveCursor<&mut CdRomCursor<T>>), io::Error> {
		match DriveCursor::new(&mut self.cdrom, Self::F_OFFSET, Self::F_SIZE) {
			Ok(cursor) => Ok((&mut self.f_drv_cursor, cursor)),
			Err(err) => Err(err),
		}
	}

	/// Returns the `G.DRV` file alongside it's cursor
	pub fn g_drv(&mut self) -> Result<(&mut DrvFsCursor, DriveCursor<&mut CdRomCursor<T>>), io::Error> {
		match DriveCursor::new(&mut self.cdrom, Self::G_OFFSET, Self::G_SIZE) {
			Ok(cursor) => Ok((&mut self.g_drv_cursor, cursor)),
			Err(err) => Err(err),
		}
	}

	/// Returns the `P.DRV` file alongside it's cursor
	pub fn p_drv(&mut self) -> Result<(&mut DrvFsCursor, DriveCursor<&mut CdRomCursor<T>>), io::Error> {
		match DriveCursor::new(&mut self.cdrom, Self::P_OFFSET, Self::P_SIZE) {
			Ok(cursor) => Ok((&mut self.p_drv_cursor, cursor)),
			Err(err) => Err(err),
		}
	}
}

// Files
impl<T: io::Seek + io::Read> GameFile<T> {
	/// Opens a file
	pub fn open_file(&mut self, path: &Path) -> Result<OpenFile<DriveCursor<&mut CdRomCursor<T>>>, OpenFileError> {
		// Check the drive we're accessing.
		let (drive, path) = path.drive().ok_or(OpenFileError::NoDrive)?;
		let (cursor, drive) = match drive.as_char() {
			'A' => self.a_drv().map_err(OpenFileError::OpenDrive)?,
			'B' => self.b_drv().map_err(OpenFileError::OpenDrive)?,
			'C' => self.c_drv().map_err(OpenFileError::OpenDrive)?,
			'E' => self.e_drv().map_err(OpenFileError::OpenDrive)?,
			'F' => self.f_drv().map_err(OpenFileError::OpenDrive)?,
			'G' => self.g_drv().map_err(OpenFileError::OpenDrive)?,
			'P' => self.p_drv().map_err(OpenFileError::OpenDrive)?,
			drive => return Err(OpenFileError::UnknownDrive { drive }),
		};

		// Then get the path from the drive
		cursor.open_file(drive, path.as_str()).map_err(OpenFileError::OpenFile)
	}
}

/// Driver cursor
pub type DriveCursor<T> = IoCursor<T>;
