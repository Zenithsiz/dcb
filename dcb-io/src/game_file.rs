//! Abstraction over the game file.
//!
//! See [`GameFile`] for details

// Modules
pub mod error;

// Exports
pub use error::NewError;

// Imports
use crate::DrvFs;
use dcb_cdrom_xa::CdRom;
use std::io;

/// Game file reader.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct GameFile {
	/// A.DRV filesystem
	pub a_drv: DrvFs,

	/// B.DRV filesystem
	pub b_drv: DrvFs,

	/// C.DRV filesystem
	pub c_drv: DrvFs,

	/// E.DRV filesystem
	pub e_drv: DrvFs,

	/// F.DRV filesystem
	pub f_drv: DrvFs,

	/// G.DRV filesystem
	pub g_drv: DrvFs,

	/// P.DRV filesystem
	pub p_drv: DrvFs,
}

// Constructors
impl GameFile {
	/// Creates a new game file from the cd reader
	pub fn new<R: io::Read + io::Seek>(cdrom: &mut CdRom<R>) -> Result<Self, NewError> {
		// Read the filesystem
		let filesystem = dcb_iso9660::Filesystem::new(cdrom).map_err(NewError::NewIso9660FileSystem)?;

		// Read all the files we care about
		let _entries = filesystem
			.root_dir()
			.read_entries(cdrom)
			.map_err(NewError::Iso9660FilesystemRootReadEntries)?;

		/*
		let _a_drv_entry = dcb_iso9660::Entry::search_entries(&entries, "A.DRV;1").ok_or(NewError::Iso9660FilesystemFindFileA)?;
		let _b_drv_entry = dcb_iso9660::Entry::search_entries(&entries, "B.DRV;1").ok_or(NewError::Iso9660FilesystemFindFileB)?;
		let _c_drv_entry = dcb_iso9660::Entry::search_entries(&entries, "C.DRV;1").ok_or(NewError::Iso9660FilesystemFindFileC)?;
		let _e_drv_entry = dcb_iso9660::Entry::search_entries(&entries, "E.DRV;1").ok_or(NewError::Iso9660FilesystemFindFileE)?;
		let _f_drv_entry = dcb_iso9660::Entry::search_entries(&entries, "F.DRV;1").ok_or(NewError::Iso9660FilesystemFindFileF)?;
		let _g_drv_entry = dcb_iso9660::Entry::search_entries(&entries, "G.DRV;1").ok_or(NewError::Iso9660FilesystemFindFileG)?;
		let _p_drv_entry = dcb_iso9660::Entry::search_entries(&entries, "P.DRV;1").ok_or(NewError::Iso9660FilesystemFindFileP)?;
		*/

		todo!();

		/*

		let a_drv_bytes = a_drv_entry.read(cdrom).map_err(NewError::Iso9660FilesystemReadFileA)?;
		let a_drv = DrvFs::from_bytes(&a_drv_bytes).map_err(NewError::ParseFilesystemA)?;

		let b_drv_entry = dcb_iso9660::Entry::search_entries(&entries, "B.DRV;1").ok_or(NewError::Iso9660FilesystemFindFileB)?;
		let b_drv_bytes = b_drv_entry.read(cdrom).map_err(NewError::Iso9660FilesystemReadFileB)?;
		let b_drv = DrvFs::from_bytes(&b_drv_bytes).map_err(NewError::ParseFilesystemB)?;

		let c_drv_entry = dcb_iso9660::Entry::search_entries(&entries, "C.DRV;1").ok_or(NewError::Iso9660FilesystemFindFileC)?;
		let c_drv_bytes = c_drv_entry.read(cdrom).map_err(NewError::Iso9660FilesystemReadFileC)?;
		let c_drv = DrvFs::from_bytes(&c_drv_bytes).map_err(NewError::ParseFilesystemC)?;

		let e_drv_entry = dcb_iso9660::Entry::search_entries(&entries, "E.DRV;1").ok_or(NewError::Iso9660FilesystemFindFileE)?;
		let e_drv_bytes = e_drv_entry.read(cdrom).map_err(NewError::Iso9660FilesystemReadFileE)?;
		let e_drv = DrvFs::from_bytes(&e_drv_bytes).map_err(NewError::ParseFilesystemE)?;

		let f_drv_entry = dcb_iso9660::Entry::search_entries(&entries, "F.DRV;1").ok_or(NewError::Iso9660FilesystemFindFileF)?;
		let f_drv_bytes = f_drv_entry.read(cdrom).map_err(NewError::Iso9660FilesystemReadFileF)?;
		let f_drv = DrvFs::from_bytes(&f_drv_bytes).map_err(NewError::ParseFilesystemF)?;

		let g_drv_entry = dcb_iso9660::Entry::search_entries(&entries, "G.DRV;1").ok_or(NewError::Iso9660FilesystemFindFileG)?;
		let g_drv_bytes = g_drv_entry.read(cdrom).map_err(NewError::Iso9660FilesystemReadFileG)?;
		let g_drv = DrvFs::from_bytes(&g_drv_bytes).map_err(NewError::ParseFilesystemG)?;

		let p_drv_entry = dcb_iso9660::Entry::search_entries(&entries, "P.DRV;1").ok_or(NewError::Iso9660FilesystemFindFileP)?;
		let p_drv_bytes = p_drv_entry.read(cdrom).map_err(NewError::Iso9660FilesystemReadFileP)?;
		let p_drv = DrvFs::from_bytes(&p_drv_bytes).map_err(NewError::ParseFilesystemP)?;

		Ok(Self {
			a_drv,
			b_drv,
			c_drv,
			e_drv,
			f_drv,
			g_drv,
			p_drv,
		})
		*/
	}
}
