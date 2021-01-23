#![doc(include = "lib.md")]
// Features
#![feature(
	never_type,
	stmt_expr_attributes,
	unwrap_infallible,
	format_args_capture,
	min_const_generics,
	array_methods,
	array_value_iter,
	external_doc
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
#![allow(clippy::match_bool)]
// If the `else` isn't needed, we don't put it
#![allow(clippy::else_if_without_else)]


// Modules
pub mod error;
pub mod iter;
pub mod sector;

// Exports
pub use error::{ReadNthSectorError, ReadSectorError, SeekSectorError};
pub use iter::SectorsRangeIter;
pub use sector::Sector;

// Imports
use dcb_bytes::Bytes;
use std::io::{Read, Seek, SeekFrom};

/// A CD-ROM/XA Mode 2 Form 1 wrapper
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct CdRom<R> {
	/// Underlying reader
	reader: R,
}

// Constants
impl<R> CdRom<R> {
	/// Sector size
	pub const SECTOR_SIZE: u64 = 2352;
}

// Constructors
impl<R> CdRom<R> {
	/// Creates a new CD-ROM reader
	#[must_use]
	pub const fn new(reader: R) -> Self {
		Self { reader }
	}
}

// Read
impl<R: Read> CdRom<R> {
	/// Reads the next sector
	pub fn read_sector(&mut self) -> Result<Sector, ReadSectorError> {
		// Read it
		let mut bytes = [0; 2352];
		self.reader.read_exact(&mut bytes).map_err(ReadSectorError::Read)?;

		// And parse it
		Sector::from_bytes(&bytes).map_err(ReadSectorError::Parse)
	}

	/// Returns an iterator over the next sectors
	pub fn read_sectors(&mut self) -> SectorsRangeIter<R> {
		SectorsRangeIter::new(self)
	}
}

// Seek
impl<R: Seek> CdRom<R> {
	/// Seeks to the `n`th sector
	pub fn seek_sector(&mut self, n: u64) -> Result<(), SeekSectorError> {
		// Seek to the sector.
		match self.reader.seek(SeekFrom::Start(Self::SECTOR_SIZE * n)) {
			Ok(_) => Ok(()),
			Err(err) => Err(SeekSectorError { sector: n, err }),
		}
	}
}

// Seek + Read
impl<R: Read + Seek> CdRom<R> {
	/// Reads the `n`th sector
	pub fn read_nth_sector(&mut self, n: u64) -> Result<Sector, ReadNthSectorError> {
		// Seek to the sector.
		self.seek_sector(n).map_err(ReadNthSectorError::Seek)?;

		// Then read it.
		self.read_sector().map_err(ReadNthSectorError::ReadNext)
	}
}
