//! Sector subheader
//!
//! # Documentation
//! All documentation in this module is from the Green Book (May 1994, Release 2).

// Modules
pub mod error;
pub mod submode;

// Exports
pub use error::{FromBytesError, ToBytesError};
pub use submode::SubMode;

// Imports
use dcb_bytes::Bytes;
use dcb_util::{array_split, array_split_mut};

/// The sector sub-header
///
/// `II.4.5`
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct SubHeader {
	/// File number.
	///
	/// Identifies all sectors that belong to one and the same file.
	///
	/// `II.4.5.2.1`
	pub file: u8,

	/// Audio channel
	///
	/// A real-time record may contain several different pieces of information
	/// that need to be chosen in combination or separately at playback.
	///
	/// To facilitate the real-time selection of such information each piece of
	/// information may be given a unique channel number.
	///
	/// `II.4.5.2.2`
	pub channel: u8,

	/// Submode
	///
	/// The submode byte defines global attributes of a sector as required for the initial
	/// selection and allocation of a sector in the system, termination of a file or record,
	/// initialization of an additional layer of error correction, and synchronization.
	///
	/// `II.4.5.2.3`
	pub submode: SubMode,

	/// Coding info
	///
	/// This byte defines the details of the type of data located in the user area of the sector.
	///
	/// `II.4.5.2.4`
	pub coding_info: u8,
}


#[allow(clippy::ptr_offset_with_cast)]
impl Bytes for SubHeader {
	type ByteArray = [u8; 4];
	type FromError = FromBytesError;
	type ToError = ToBytesError;

	fn from_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::FromError> {
		let bytes = array_split!(bytes,
			file       : 0x1,
			channel    : 0x1,
			submode    : 0x1,
			coding_info: 0x1,
		);

		Ok(Self {
			file:        *bytes.file,
			channel:     *bytes.channel,
			submode:     SubMode::from_bytes(bytes.submode).map_err(FromBytesError::SubMode)?,
			coding_info: *bytes.coding_info,
		})
	}

	fn to_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::ToError> {
		let bytes = array_split_mut!(bytes,
			file       : 0x1,
			channel    : 0x1,
			submode    : 0x1,
			coding_info: 0x1,
		);

		*bytes.file = self.file;
		*bytes.channel = self.channel;
		self.submode.to_bytes(bytes.submode).map_err(ToBytesError::SubMode)?;
		*bytes.coding_info = self.coding_info;

		Ok(())
	}
}
