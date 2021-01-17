//! Game file filesystem
//!
//! The filesystem used is ISO 9960, which is mostly implemented
//! in this module, abstracted from the CD-ROM/XA sectors.

// Modules
pub mod date_time;
pub mod error;
pub mod string;
pub mod volume_descriptor;

// Exports
pub use error::NewError;
pub use string::{StrArrA, StrArrD};
pub use volume_descriptor::VolumeDescriptor;

// Imports
use crate::GameFile;
use dcb_bytes::Bytes;
use std::io;

/// The filesystem
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Filesystem {
	/// Primary volume descriptor
	primary_volume_descriptor: VolumeDescriptor,
}

impl Filesystem {
	/// Reads the filesystem from a game file
	pub fn new<R: io::Read + io::Seek>(file: &mut GameFile<R>) -> Result<Self, NewError> {
		// Read the primary volume descriptor from sector `0x10`
		let sector = file.sector(0x10).map_err(NewError::ReadPrimaryVolumeSector)?;
		let primary_volume_descriptor = VolumeDescriptor::from_bytes(&sector.data).map_err(NewError::ParsePrimaryVolume)?;

		Ok(Self { primary_volume_descriptor })
	}
}
