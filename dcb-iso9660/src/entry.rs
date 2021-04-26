//! An entry

// Modules
pub mod error;
pub mod file;

// Exports
pub use error::{FromReaderError, ReadDirError, ReadFileError, ToWriterError};
pub use file::FileReader;

// Imports
use super::string::FileString;
use crate::Dir;
use byteorder::{BigEndian, ByteOrder, LittleEndian};
use dcb_cdrom_xa::CdRomReader;
use dcb_util::{array_split, array_split_mut};
use std::{
	convert::{TryFrom, TryInto},
	io,
};

/// A directory entry.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct DirEntry {
	/// Entry's name
	pub name: FileString,

	/// Entry's location
	pub sector_pos: u32,

	/// Entry's size
	pub size: u32,

	/// Entry flags
	pub flags: Flags,
}

bitflags::bitflags! {
	pub struct Flags: u8 {
		const HIDDEN     = 0b0000_0001;
		const DIR        = 0b0000_0010;
		const ASSOCIATED = 0b0000_0100;
		const RECORD     = 0b0000_1000;
		const PROTECTED  = 0b0001_0000;
		const FINAL      = 0b1000_0000;
	}
}

impl DirEntry {
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

	/// Reads a file from this entry
	pub fn read_file<'a, R: io::Read + io::Seek>(
		&self, cdrom: &'a mut CdRomReader<R>,
	) -> Result<FileReader<'a, R>, ReadFileError> {
		// If this isn't a file, return Err
		if !self.is_file() {
			return Err(ReadFileError::NotAFile);
		}

		// Seek the cdrom to the correct place
		let sector_pos = u64::from(self.sector_pos);
		cdrom.seek_sector(sector_pos).map_err(ReadFileError::SeekSector)?;

		// And crate the file reader
		let size = u64::from(self.size);
		Ok(FileReader::new(cdrom, sector_pos, size))
	}

	/// Reads a directory from this entry.
	pub fn read_dir<R: io::Read + io::Seek>(&self, cdrom: &mut CdRomReader<R>) -> Result<Dir, ReadDirError> {
		// If this isn't a directory, return Err
		if !self.is_dir() {
			return Err(ReadDirError::NotADirectory);
		}

		// We don't currently support directories larger than a sector
		let size = usize::try_from(self.size).expect("Directory size didn't fit into a `usize`");
		if size > 2048 {
			todo!("Directory sizes larger than a sector are not supported yet.");
		}

		// Read the sector
		let sector = cdrom
			.read_nth_sector(u64::from(dbg!(self.sector_pos)))
			.map_err(ReadDirError::ReadSector)?;

		// Then keep parsing until we run out.
		let data = sector.data.as_form1().ok_or(ReadDirError::DirSectorWrongForm)?;
		let mut bytes = std::io::Cursor::new(&data[..size]);
		let dirs = std::iter::from_fn(move || match Self::from_reader(&mut bytes) {
			// Note: If it fails due to the record size being 0, return None
			Err(FromReaderError::RecordSizeTooSmall(0)) => None,
			Ok(entry) => {
				// If it's the current or parent directory, skip
				if *entry.name == [b'\x00'] || *entry.name == [b'\x01'] {
					Some(Ok(None))
				} else {
					Some(Ok(Some(entry)))
				}
			},
			Err(err) => Some(Err(ReadDirError::ParseEntry(err))),
		})
		.flat_map(Result::transpose)
		.collect::<Result<Vec<_>, _>>()?;

		Ok(Dir::new(dirs))
	}
}

impl DirEntry {
	/// Reads a directory entry from a reader
	pub fn from_reader<R: io::Read>(reader: &mut R) -> Result<Self, FromReaderError> {
		// Get the header
		let mut header_bytes = [0; 0x21];
		reader
			.read_exact(&mut header_bytes)
			.map_err(FromReaderError::ReadHeader)?;

		let header_bytes = array_split!(&header_bytes,
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
			return Err(FromReaderError::RecordSizeTooSmall(*header_bytes.record_size));
		}

		// If this is interleaved, we don't support it yet
		if *header_bytes.file_unit_size != 0 || *header_bytes.interleave_gap_size != 0 {
			todo!("Interleaved entries aren't supported yet");
		}

		// Then read the name
		// TODO: Avoid double allocation by having `FileString` consume `name_bytes`
		let mut name_bytes = vec![0; usize::from(*header_bytes.name_len)];
		reader.read_exact(&mut name_bytes).map_err(FromReaderError::ReadName)?;
		let name = FileString::from_bytes(&name_bytes).map_err(FromReaderError::ParseName)?;

		// Then skip the remaining bytes
		let mut remaining = vec![0; usize::from(header_bytes.record_size - 0x21 - header_bytes.name_len)];
		reader
			.read_exact(&mut remaining)
			.map_err(FromReaderError::ReadRemaining)?;

		Ok(Self {
			name,
			sector_pos: LittleEndian::read_u32(header_bytes.extent_location_lsb),
			size: LittleEndian::read_u32(header_bytes.extent_size_lsb),
			flags: Flags::from_bits(*header_bytes.file_flags).ok_or(FromReaderError::InvalidFlags)?,
		})
	}

	/// Writes this directory to a writer
	pub fn to_writer<W: io::Write>(&self, writer: &mut W) -> Result<(), ToWriterError> {
		// Write the header
		let mut header_bytes = [0u8; 0x21];
		let header = array_split_mut!(&mut header_bytes,
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

		// Fill the header
		*header.record_size = (0x21 + self.name.len()).try_into().expect("Name was too large");
		*header.extended_attribute_record_len = 0;
		LittleEndian::write_u32(header.extent_location_lsb, self.sector_pos);
		BigEndian::write_u32(header.extent_location_lsb, self.sector_pos);
		LittleEndian::write_u32(header.extent_size_lsb, self.size);
		BigEndian::write_u32(header.extent_size_msb, self.size);
		*header.file_flags = self.flags.bits();
		*header.file_unit_size = 0;
		*header.interleave_gap_size = 0;
		*header.name_len = *header.record_size - 0x21;

		// Write the header
		writer.write_all(&header_bytes).map_err(ToWriterError::WriteHeader)?;

		// Then write the name
		writer
			.write_all(self.name.as_bytes())
			.map_err(ToWriterError::WriteName)?;

		Ok(())
	}
}
