#![doc(include = "fs.md")]

// Modules
pub mod date_time;
pub mod entry;
pub mod error;
pub mod string;
pub mod volume_descriptor;

// Exports
pub use entry::Entry;
pub use error::NewError;
pub use string::{StrArrA, StrArrD};
pub use volume_descriptor::VolumeDescriptor;

// Imports
use self::volume_descriptor::PrimaryVolumeDescriptor;
use crate::CdRom;
use dcb_bytes::Bytes;
use std::io;

/// The filesystem
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Filesystem {
	// TODO: Only read the root directory and necessary information
	//       to reconstruct the filesystem.
	/// Primary volume descriptor
	primary_volume_descriptor: PrimaryVolumeDescriptor,
}

impl Filesystem {
	/// Reads the filesystem from the cd rom.
	pub fn new<R: io::Read + io::Seek>(file: &mut CdRom<R>) -> Result<Self, NewError> {
		// Start reading volume descriptors from sector `0x10` until we hit the primary one
		// Note: First `32 kiB` (= 16 sectors) are reserved for arbitrary data.
		let mut sectors = file.read_sectors_range(0x10..);
		let primary_volume_descriptor = loop {
			match sectors.next() {
				Some(Ok(sector)) => match VolumeDescriptor::from_bytes(&sector.data) {
					Ok(VolumeDescriptor::Primary(primary)) => break primary,
					Ok(VolumeDescriptor::SetTerminator) => return Err(NewError::MissingPrimaryVolumeBeforeSetTerminator),
					Ok(volume_descriptor) => log::debug!("Skipping {:?} volume descriptor before primary", volume_descriptor.kind()),
					Err(err) => return Err(NewError::InvalidVolumeDescriptor(err)),
				},
				Some(Err(err)) => return Err(NewError::InvalidSectorBeforeSetTerminator(err)),
				None => return Err(NewError::EofBeforeSetTerminator),
			}
		};

		Ok(Self { primary_volume_descriptor })
	}

	/// Returns the root directory entry
	#[must_use]
	pub const fn root_dir(&self) -> &Entry {
		&self.primary_volume_descriptor.root_dir_entry
	}
}
