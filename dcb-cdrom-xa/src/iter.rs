//! Iterators

// Imports
use super::{ReadSectorError, Sector};
use crate::CdRom;
use std::io;

/// Iterator over sectors
pub struct SectorsRangeIter<'a, R: io::Read> {
	/// The cdrom
	cdrom: &'a mut CdRom<R>,
}

impl<'a, R: io::Read> SectorsRangeIter<'a, R> {
	/// Creates a new sector range iterator
	pub(crate) fn new(cdrom: &'a mut CdRom<R>) -> Self {
		Self { cdrom }
	}
}


impl<'a, R: io::Read> Iterator for SectorsRangeIter<'a, R> {
	type Item = Result<Sector, ReadSectorError>;

	fn next(&mut self) -> Option<Self::Item> {
		// Read the next sector
		Some(self.cdrom.read_sector())
	}
}
