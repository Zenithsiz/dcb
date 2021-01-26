//! Iterators

// Imports
use crate::{reader, CdRomReader, Sector};
use std::io;

/// Iterator over sectors
pub struct SectorsRangeIter<'a, R: io::Read> {
	/// The cdrom
	cdrom: &'a mut CdRomReader<R>,
}

impl<'a, R: io::Read> SectorsRangeIter<'a, R> {
	/// Creates a new sector range iterator
	pub(crate) fn new(cdrom: &'a mut CdRomReader<R>) -> Self {
		Self { cdrom }
	}
}


impl<'a, R: io::Read> Iterator for SectorsRangeIter<'a, R> {
	type Item = Result<Sector, reader::ReadSectorError>;

	fn next(&mut self) -> Option<Self::Item> {
		// Read the next sector
		Some(self.cdrom.read_sector())
	}
}
