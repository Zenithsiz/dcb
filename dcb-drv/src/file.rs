//! File

// Imports
use dcb_util::AsciiStrArr;
use std::io;

/// A file writer
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct FileWriter<R: io::Read> {
	/// File extension
	pub extension: AsciiStrArr<0x3>,

	/// Reader
	pub reader: R,

	/// Size
	pub size: u32,
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
