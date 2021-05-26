//! Directory entry writer

// Imports
use crate::{DirWriter, DirWriterLister, FileWriter};
use chrono::NaiveDateTime;
use dcb_util::AsciiStrArr;
use std::fmt;

/// A directory entry writer kind
pub enum DirEntryWriterKind<L: DirWriterLister> {
	/// A file
	File(FileWriter<L::FileReader>),

	/// Directory
	Dir(DirWriter<L>),
}

impl<L: DirWriterLister + fmt::Debug> fmt::Debug for DirEntryWriterKind<L>
where
	L::FileReader: std::fmt::Debug,
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::File(file) => f.debug_tuple("File").field(file).finish(),
			Self::Dir(dir) => f.debug_tuple("Dir").field(dir).finish(),
		}
	}
}

/// A directory entry writer
pub struct DirEntryWriter<L: DirWriterLister> {
	/// Entry name
	pub name: AsciiStrArr<0x10>,

	/// Entry date
	pub date: NaiveDateTime,

	/// Entry kind
	pub kind: DirEntryWriterKind<L>,
}

impl<L: DirWriterLister + fmt::Debug> fmt::Debug for DirEntryWriter<L>
where
	L::FileReader: std::fmt::Debug,
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("DirEntryWriter")
			.field("name", &self.name)
			.field("date", &self.date)
			.field("kind", &self.kind)
			.finish()
	}
}

impl<L: DirWriterLister> DirEntryWriter<L> {
	/// Creates a new entry writer from it's name, date and kind
	pub fn new(name: AsciiStrArr<0x10>, date: NaiveDateTime, kind: DirEntryWriterKind<L>) -> Self {
		Self { name, date, kind }
	}

	/// Returns this entry's size
	///
	/// For directories this simply returns the directory size itself,
	/// _not_ the sum of the sizes of it's entries.
	pub fn size(&self) -> u32 {
		match &self.kind {
			DirEntryWriterKind::File(file) => file.size(),
			DirEntryWriterKind::Dir(dir) => dir.size(),
		}
	}
}
