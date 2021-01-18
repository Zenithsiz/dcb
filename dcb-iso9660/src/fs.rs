//! ISO 9660 filesystem.
//!
//! This module implements the ISO 9660 filesystem.

// Modules
pub mod date_time;
pub mod dir_record;
pub mod error;
pub mod string;
pub mod volume_descriptor;

// Exports
pub use dir_record::DirRecord;
pub use error::NewError;
pub use string::{StrArrA, StrArrD};
pub use volume_descriptor::VolumeDescriptor;

// Imports
use crate::CdRom;
use dcb_bytes::Bytes;
use std::io;

/// The filesystem
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Filesystem {
	/// Root directory
	root_dir_entry: DirRecord,
}

impl Filesystem {
	/// Reads the filesystem from a game file
	pub fn new<R: io::Read + io::Seek>(file: &mut CdRom<R>) -> Result<Self, NewError> {
		// Read the primary volume descriptor from sector `0x10`
		// Note: First `32 kiB` (= 16 sectors) are reserved for arbitrary data.
		let sector = file.sector(0x10).map_err(NewError::ReadPrimaryVolumeSector)?;
		let _primary_volume_descriptor = VolumeDescriptor::from_bytes(&sector.data).map_err(NewError::ParsePrimaryVolume)?;

		todo!();

		/*
		// Try to get the root directory entry
		let root_dir_entry = match primary_volume_descriptor {
			VolumeDescriptor::Primary { root_dir_entry, .. } => root_dir_entry,
			_ => return Err(NewError::FirstVolumeNotPrimary(primary_volume_descriptor.type_code())),
		};

		Ok(Self { root_dir_entry })
		*/
	}

	/// Prints a tree of all files
	pub fn fmt_tree<R: io::Read + io::Seek>(&self, _cdrom: &mut CdRom<R>) {
		let _ = self;
		todo!();
	}
}
