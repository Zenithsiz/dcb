//! An entry

// Modules
pub mod error;

// Exports
pub use error::{FromBytesError, ReadEntriesError, ReadError};

// Imports
use crate::CdRom;

use super::string::FileString;
use byteorder::{ByteOrder, LittleEndian};
use dcb_bytes::Bytes;
use dcb_util::array_split;
use std::{convert::TryInto, io};

/// An entry
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Entry {
	/// Entry's name
	name: FileString,

	/// Entry's location
	location: u32,

	/// Entry's size
	size: u32,

	/// Entry flags
	flags: Flags,
}

bitflags::bitflags! {
	struct Flags: u8 {
		const HIDDEN = 0b0000_0001;
		const DIR    = 0b0000_0010;
	}
}

impl Entry {
	/// Returns if this entry is a directory
	#[must_use]
	pub const fn is_dir(&self) -> bool {
		self.flags.contains(Flags::DIR)
	}

	/// Returns if this entry is a file
	#[must_use]
	pub const fn is_file(&self) -> bool {
		!self.is_dir()
	}

	/// Finds an entry in a list of entries
	#[must_use]
	pub fn search_entries<'a>(entries: &'a [Self], name: &str) -> Option<&'a Self> {
		for entry in entries {
			// TODO: Avoid allocation
			if entry.name.to_string() == name {
				return Some(entry);
			}
		}

		None
	}

	/// Reads this file
	pub fn read<R: io::Read + io::Seek>(&self, cdrom: &mut CdRom<R>) -> Result<Vec<u8>, ReadError> {
		// If this isn't a file, return Err
		if !self.is_file() {
			return Err(ReadError::NotAFile);
		}

		let start = u64::from(self.location);
		let end = start + u64::from(self.size) / 2048;
		let remaining = self.size % 2048;

		// Read all full sectors
		// TODO: Avoid double allocation here
		let mut bytes: Vec<u8> = cdrom
			.read_sectors_range(start..end)
			.map(|res| res.map(|sector| sector.data).map(std::array::IntoIter::new))
			.collect::<Result<Vec<_>, _>>()
			.map_err(ReadError::ReadSector)?
			.into_iter()
			.flatten()
			.collect();

		// Then read the remaining sector
		if remaining != 0 {
			let last_sector = cdrom.read_sector(end).map_err(ReadError::ReadSector)?;
			#[allow(clippy::as_conversions)] // `remaining < 2048`
			bytes.extend(&last_sector.data[..remaining as usize]);
		}

		Ok(bytes)
	}

	/// Reads all entries in this entry, if a directory
	pub fn read_entries<R: io::Read + io::Seek>(&self, cdrom: &mut CdRom<R>) -> Result<Vec<Self>, ReadEntriesError> {
		// If this isn't a directory, return Err
		if !self.is_dir() {
			return Err(ReadEntriesError::NotADirectory);
		}

		// We don't currently support directories larger than a sector
		if self.size > 2048 {
			todo!("Directory sizes larger than a sector are not supported yet.");
		}

		// Read the sector
		let sector = cdrom.read_sector(u64::from(self.location)).map_err(ReadEntriesError::ReadSector)?;

		// Then keep parsing until we hit our size
		let mut dirs = vec![];
		let mut cur_pos = 0;
		#[allow(clippy::as_conversions)] // We checked `size <= 2048`
		while cur_pos < (self.size as usize) {
			// Get the bytes for this entry
			let bytes = &sector.data[cur_pos..];

			// Get the entry's length, if it's 0, break
			let dir_size = usize::from(bytes[0]);
			if dir_size == 0 {
				break;
			}

			// Read the entry then skip it's length
			let dir = Self::from_bytes(bytes).map_err(ReadEntriesError::ParseEntry)?;
			dirs.push(dir);
			cur_pos += dir_size;
		}

		Ok(dirs)
	}
}

impl Bytes for Entry {
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

		// If the record size isn't at least `0x21` + `name_len`, return Err
		if *header_bytes.record_size < 0x21 + header_bytes.name_len {
			return Err(FromBytesError::RecordSizeTooSmall);
		}

		// If this is interleaved, we don't support it yet
		if *header_bytes.file_unit_size != 0 || *header_bytes.interleave_gap_size != 0 {
			todo!("Interleaved entries aren't supported yet");
		}

		// Then read the name
		let name = bytes
			.get(0x21..0x21 + usize::from(*header_bytes.name_len))
			.ok_or(FromBytesError::TooSmallName(*header_bytes.name_len))?;
		let name = FileString::from_bytes(name).map_err(FromBytesError::Name)?;

		Ok(Self {
			name,
			location: LittleEndian::read_u32(header_bytes.extent_location_lsb),
			size: LittleEndian::read_u32(header_bytes.extent_size_lsb),
			flags: Flags::from_bits(*header_bytes.file_flags).ok_or(FromBytesError::InvalidFlags)?,
		})
	}

	fn to_bytes(&self, _bytes: &mut Self::ByteArray) -> Result<(), Self::ToError> {
		todo!()
	}
}
