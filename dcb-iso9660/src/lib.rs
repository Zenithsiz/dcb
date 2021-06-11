#![doc = include_str!("lib.md")]
// Features
#![feature(
	never_type,
	unwrap_infallible,
	format_args_capture,
	array_methods,
	str_internals
)]
// Lints
#![warn(clippy::restriction, clippy::pedantic, clippy::nursery)]
// We'll disable the ones we don't need
#![allow(clippy::blanket_clippy_restriction_lints)]
// No unsafe allowed in this crate
#![forbid(unsafe_code)]
// Must use `expect` instead of `unwrap`
#![forbid(clippy::unwrap_used)]
// We don't need to mark every public function `inline`
#![allow(clippy::missing_inline_in_public_items)]
// We prefer literals to be copy-paste-able rather than readable
#![allow(clippy::unreadable_literal)]
// We prefer suffixes to be glued to the literal
#![allow(clippy::unseparated_literal_suffix)]
// We're fine with panicking when entering an unexpected state
#![allow(
	clippy::panic,
	clippy::unreachable,
	clippy::expect_used,
	clippy::panic_in_result_fn,
	clippy::unwrap_in_result,
	clippy::indexing_slicing,
	clippy::todo
)]
// We prefer tail calls
#![allow(clippy::implicit_return)]
// We use multiple implementations to separate logic
#![allow(clippy::multiple_inherent_impl)]
// We use granular error types, usually one for each function, which document the
// errors that might happen, as opposed to documenting them in the function
#![allow(clippy::missing_errors_doc)]
// Due to our module organization, we end up with data types inheriting their module's name
#![allow(clippy::module_name_repetitions)]
// We need arithmetic for this crate
#![allow(clippy::integer_arithmetic, clippy::integer_division)]
// We want to benefit from match ergonomics where possible
#![allow(clippy::pattern_type_mismatch)]
// We only use wildcards when we only care about certain variants
#![allow(clippy::wildcard_enum_match_arm, clippy::match_wildcard_for_single_variants)]
// We're fine with shadowing, as long as it's related
#![allow(clippy::shadow_reuse, clippy::shadow_same)]
// Matching on booleans can look better than `if / else`
#![allow(clippy::match_bool, clippy::single_match_else, clippy::if_not_else)]
// If the `else` isn't needed, we don't put it
#![allow(clippy::else_if_without_else)]
// We're fine with non-exhaustive structs / enums, we aren't committing to them yet.
#![allow(clippy::exhaustive_structs, clippy::exhaustive_enums)]
// There are too many false positives with these lints
#![allow(clippy::use_self)]
// `Header` and `Reader` are common names
#![allow(clippy::similar_names)]
// We only use `# Panics` where a panic might be caused by a mis-use of the user, not assertions
#![allow(clippy::missing_panics_doc)]
// Some errors don't carry enough information to include them in another super-error
#![allow(clippy::map_err_ignore)]


// Modules
pub mod date_time;
pub mod dir;
pub mod entry;
pub mod error;
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
