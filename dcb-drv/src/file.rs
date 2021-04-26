//! File


// Imports
use dcb_util::AsciiStrArr;
use std::io::{self, SeekFrom};

/// A file reader
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct FileReader {
	/// File extension
	extension: AsciiStrArr<0x3>,

	/// Sector position
	sector_pos: u32,

	/// Size
	size: u32,
}

impl FileReader {
	/// Creates a new file reader from it's extension, sector position and size.
	#[must_use]
	pub const fn new(extension: AsciiStrArr<0x3>, sector_pos: u32, size: u32) -> Self {
		Self {
			extension,
			sector_pos,
			size,
		}
	}

	/// Returns this file's extension
	#[must_use]
	pub const fn extension(&self) -> &AsciiStrArr<0x3> {
		&self.extension
	}

	/// Returns this file's sector position
	#[must_use]
	pub const fn sector_pos(&self) -> u32 {
		self.sector_pos
	}

	/// Returns this file's sector size
	#[must_use]
	pub const fn size(&self) -> u32 {
		self.size
	}

	/// Returns a reader for this file from the filesystem reader
	pub fn reader<'a, R: io::Read + io::Seek>(&self, reader: &'a mut R) -> Result<impl io::Read + 'a, io::Error> {
		// Seek to our file
		self.seek_to(reader)?;

		// Then take at most our size
		let reader = <&mut R as io::Read>::take(reader, u64::from(self.size));
		Ok(reader)
	}

	/// Seeks to this file on a reader
	pub fn seek_to<R: io::Seek>(&self, reader: &mut R) -> Result<u64, io::Error> {
		reader.seek(SeekFrom::Start(u64::from(self.sector_pos) * 2048))
	}
}

/// A file writer
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct FileWriter<R: io::Read> {
	/// File extension
	extension: AsciiStrArr<0x3>,

	/// Reader
	reader: R,

	/// Size
	size: u32,
}

impl<R: io::Read> FileWriter<R> {
	/// Creates a new file writer from it's extension and reader.
	pub fn new(extension: AsciiStrArr<0x3>, reader: R, size: u32) -> Self {
		Self {
			extension,
			reader,
			size,
		}
	}

	/// Returns this file's extension
	#[must_use]
	pub fn extension(&self) -> &AsciiStrArr<0x3> {
		&self.extension
	}

	/// Returns this file's size
	pub fn size(&self) -> u32 {
		self.size
	}

	/// Writes this file to a writer
	pub fn write<W: io::Write>(self, writer: &mut W) -> Result<(), io::Error> {
		let written = std::io::copy(&mut self.reader.take(u64::from(self.size)), writer)?;
		assert_eq!(written, u64::from(self.size));
		Ok(())
	}
}
