//! A directory entry

// Modules
pub mod error;

// Exports
pub use error::FromBytesError;

// Imports
use super::string::FileString;
use byteorder::{ByteOrder, LittleEndian};
use dcb_bytes::Bytes;
use dcb_util::array_split;
use std::convert::TryInto;

/// A directory record
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct DirRecord {
	/// Record total size
	record_size: u8,

	/// File's name
	name: FileString,

	/// File's location
	location: u32,

	/// File's size
	size: u32,
}

impl DirRecord {
	/// Returns this record's size
	#[must_use]
	pub const fn size(&self) -> u8 {
		self.record_size
	}
}

impl Bytes for DirRecord {
	type ByteArray = [u8];
	type FromError = FromBytesError;
	type ToError = !;

	fn from_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::FromError> {
		// Get the header
		let header_bytes: &[u8; 0x21] = match bytes.get(..0x21).and_then(|bytes| bytes.try_into().ok()) {
			Some(header_bytes) => header_bytes,
			None => return Err(FromBytesError::TooSmallHeader),
		};

		let header_bytes = array_split!(header_bytes,
			record_size                  :  0x1,
			extended_attribute_record_len:  0x1,
			extent_location_lsb          : [0x4],
			extent_location_msb          : [0x4],
			extent_size_lsb              : [0x4],
			extent_size_msb              : [0x4],
			recording_date_time          : [0x7],
			file_flags                   :  0x1,
			file_unit_size               :  0x1,
			interleave_gap_size          :  0x1,
			volume_sequence_number_lsb   : [0x2],
			volume_sequence_number_msb   : [0x2],
			name_len                     :  0x1,
		);

		dbg!(bytes);

		// Then read the name
		let name = bytes
			.get(0x21..0x21 + usize::from(*header_bytes.name_len))
			.ok_or(FromBytesError::TooSmallName(*header_bytes.name_len))?;
		let name = FileString::from_bytes(name).map_err(FromBytesError::Name)?;

		Ok(Self {
			record_size: *header_bytes.record_size,
			name,
			location: LittleEndian::read_u32(header_bytes.extent_location_lsb),
			size: LittleEndian::read_u32(header_bytes.extent_size_lsb),
		})
	}

	fn to_bytes(&self, _bytes: &mut Self::ByteArray) -> Result<(), Self::ToError> {
		todo!()
	}
}
