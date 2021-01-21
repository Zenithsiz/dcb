//! Iterators

// Imports
use super::{ReadSectorError, Sector};
use crate::CdRom;
use std::{
	io,
	ops::{Bound, Range, RangeBounds},
};

/// Iterator over sectors
pub struct SectorsRangeIter<'a, R: io::Read + io::Seek> {
	/// The cdrom
	cdrom: &'a mut CdRom<R>,

	/// Sector range
	range: Range<u64>,
}

impl<'a, R: io::Read + io::Seek> SectorsRangeIter<'a, R> {
	/// Creates a new sector range iterator
	pub(crate) fn new(cdrom: &'a mut CdRom<R>, range: impl RangeBounds<u64>) -> Self {
		let start = match range.start_bound() {
			Bound::Included(&n) => n,
			Bound::Excluded(n) => n.saturating_add(1),
			Bound::Unbounded => 0,
		};
		let end = match range.end_bound() {
			Bound::Included(n) => n.saturating_add(1),
			Bound::Excluded(&n) => n,
			Bound::Unbounded => u64::MAX,
		};

		Self { cdrom, range: start..end }
	}
}


impl<'a, R: io::Read + io::Seek> Iterator for SectorsRangeIter<'a, R> {
	type Item = Result<Sector, ReadSectorError>;

	fn next(&mut self) -> Option<Self::Item> {
		// TODO: Maybe only seek once and them keep reading?

		// If our range is empty, return None
		if self.range.is_empty() {
			return None;
		}

		// Else read the next sector and advance our range
		let n = self.range.start;
		self.range.start += 1;
		Some(self.cdrom.read_sector(n))
	}
}
