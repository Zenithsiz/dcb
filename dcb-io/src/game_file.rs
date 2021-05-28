//! Game file.
//!
//! See [`GameFile`] for details

// Modules
pub mod error;
pub mod path;

// Exports
pub use path::Path;

// Imports
use dcb_cdrom_xa::CdRomCursor;
use dcb_util::IoCursor;
use std::io;

/// Game file.
#[derive(PartialEq, Clone, Debug)]
pub struct GameFile<T> {
	/// CD-Rom
	cdrom: CdRomCursor<T>,
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
	pub fn new(cdrom: CdRomCursor<T>) -> Self {
		Self { cdrom }
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
	pub fn a_drv(&mut self) -> Result<DriveCursor<&mut CdRomCursor<T>>, io::Error> {
		match DriveCursor::new(&mut self.cdrom, Self::A_OFFSET, Self::A_SIZE) {
			Ok(cursor) => Ok(cursor),
			Err(err) => Err(err),
		}
	}

	/// Returns the `B.DRV` file alongside it's cursor
	pub fn b_drv(&mut self) -> Result<DriveCursor<&mut CdRomCursor<T>>, io::Error> {
		match DriveCursor::new(&mut self.cdrom, Self::B_OFFSET, Self::B_SIZE) {
			Ok(cursor) => Ok(cursor),
			Err(err) => Err(err),
		}
	}

	/// Returns the `C.DRV` file alongside it's cursor
	pub fn c_drv(&mut self) -> Result<DriveCursor<&mut CdRomCursor<T>>, io::Error> {
		match DriveCursor::new(&mut self.cdrom, Self::C_OFFSET, Self::C_SIZE) {
			Ok(cursor) => Ok(cursor),
			Err(err) => Err(err),
		}
	}

	/// Returns the `E.DRV` file alongside it's cursor
	pub fn e_drv(&mut self) -> Result<DriveCursor<&mut CdRomCursor<T>>, io::Error> {
		match DriveCursor::new(&mut self.cdrom, Self::E_OFFSET, Self::E_SIZE) {
			Ok(cursor) => Ok(cursor),
			Err(err) => Err(err),
		}
	}

	/// Returns the `F.DRV` file alongside it's cursor
	pub fn f_drv(&mut self) -> Result<DriveCursor<&mut CdRomCursor<T>>, io::Error> {
		match DriveCursor::new(&mut self.cdrom, Self::F_OFFSET, Self::F_SIZE) {
			Ok(cursor) => Ok(cursor),
			Err(err) => Err(err),
		}
	}

	/// Returns the `G.DRV` file alongside it's cursor
	pub fn g_drv(&mut self) -> Result<DriveCursor<&mut CdRomCursor<T>>, io::Error> {
		match DriveCursor::new(&mut self.cdrom, Self::G_OFFSET, Self::G_SIZE) {
			Ok(cursor) => Ok(cursor),
			Err(err) => Err(err),
		}
	}

	/// Returns the `P.DRV` file alongside it's cursor
	pub fn p_drv(&mut self) -> Result<DriveCursor<&mut CdRomCursor<T>>, io::Error> {
		match DriveCursor::new(&mut self.cdrom, Self::P_OFFSET, Self::P_SIZE) {
			Ok(cursor) => Ok(cursor),
			Err(err) => Err(err),
		}
	}
}

/// Driver cursor
pub type DriveCursor<T> = IoCursor<T>;
