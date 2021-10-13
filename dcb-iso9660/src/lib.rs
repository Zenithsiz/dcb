#![doc = include_str!("lib.md")]
// Features
#![feature(never_type, unwrap_infallible, format_args_capture, array_methods, str_internals)]

// Modules
pub mod date_time;
pub mod dir;
pub mod entry;
mod error;
pub mod string;
pub mod volume_descriptor;

// Exports
pub use dir::Dir;
pub use entry::DirEntry;
pub use error::NewError;
pub use string::{StrArrA, StrArrD};
pub use volume_descriptor::VolumeDescriptor;

// Imports
use self::volume_descriptor::PrimaryVolumeDescriptor;
use dcb_bytes::Bytes;
use dcb_cdrom_xa::CdRomReader;
use std::io;

/// A filesystem reader
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct FilesystemReader {
	/// Primary volume descriptor
	primary_volume_descriptor: PrimaryVolumeDescriptor,
}

impl FilesystemReader {
	/// Reads the filesystem from the cd rom.
	pub fn new<R: io::Read + io::Seek>(cdrom: &mut CdRomReader<R>) -> Result<Self, NewError> {
		// Start reading volume descriptors from sector `0x10` until we hit the primary one
		// Note: First `32 kiB` (= 16 sectors) are reserved for arbitrary data.
		cdrom.seek_sector(0x10).map_err(NewError::SeekVolumeDescriptorSet)?;
		let mut sectors = cdrom.read_sectors();
		let primary_volume_descriptor = loop {
			match sectors.next() {
				Some(Ok(sector)) => {
					match VolumeDescriptor::deserialize_bytes(
						sector.data.as_form1().ok_or(NewError::PrimaryFormatWrongForm)?,
					) {
						Ok(VolumeDescriptor::Primary(primary)) => break primary,
						Ok(VolumeDescriptor::SetTerminator) => {
							return Err(NewError::MissingPrimaryVolumeBeforeSetTerminator)
						},
						Ok(volume_descriptor) => log::debug!(
							"Skipping {:?} volume descriptor before primary",
							volume_descriptor.kind()
						),
						Err(err) => return Err(NewError::InvalidVolumeDescriptor(err)),
					}
				},
				Some(Err(err)) => return Err(NewError::InvalidSectorBeforeSetTerminator(err)),
				None => return Err(NewError::EofBeforeSetTerminator),
			}
		};

		Ok(Self {
			primary_volume_descriptor,
		})
	}

	/// Returns the primary volume descriptor
	#[must_use]
	pub const fn primary_volume_descriptor(&self) -> &PrimaryVolumeDescriptor {
		&self.primary_volume_descriptor
	}

	/// Returns the root directory entry
	#[must_use]
	pub const fn root_dir(&self) -> &DirEntry {
		&self.primary_volume_descriptor.root_dir_entry
	}
}

/// A filesystem reader
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct FilesystemWriter {}

impl FilesystemWriter {
	/// Creates a new filesystem writer
	#[must_use]
	pub const fn new() -> Self {
		/*
		// Create the primary volume descriptor and write it
		// Note: W
		// 22
		let primary_volume_descriptor = PrimaryVolumeDescriptor {
			system_id: (),
			volume_id: (),
			volume_space_size: (),
			volume_sequence_number: (),
			logical_block_size: (),
			path_table_size: (),
			path_table_location: (),
			path_table_opt_location: (),
			root_dir_entry: (),
			volume_set_id: (),
			publisher_id: (),
			data_preparer_id: (),
			application_id: (),
			copyright_file_id: (),
			abstract_file_id: (),
			bibliographic_file_id: (),
			volume_creation_date_time: (),
			volume_modification_date_time: (),
			volume_expiration_date_time: (),
			volume_effective_date_time: (),
		};
		*/
		Self {}
	}
}
