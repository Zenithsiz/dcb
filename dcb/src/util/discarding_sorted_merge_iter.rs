//! Merging iterator

// Imports
use either::Either;
use std::cmp::Ordering;

/// Merging sorted iterator
///
/// Will discard duplicate items.
pub struct DiscardingSortedMergeIter<T: Ord, Li: Iterator<Item = T>, Ri: Iterator<Item = T>> {
	/// Left iterator
	lhs: Li,

	/// Right iterator
	rhs: Ri,

	/// Last element stored
	last: Option<Either<T, T>>,
}

impl<T: Ord, Li: Iterator<Item = T>, Ri: Iterator<Item = T>> DiscardingSortedMergeIter<T, Li, Ri> {
	/// Creates a new merging iterator
	#[allow(dead_code)] // TODO: Remove
	pub fn new(lhs: Li, rhs: Ri) -> Self {
		Self { lhs, rhs, last: None }
	}

	/// Chooses between two values, storing the larger one and
	/// discarding the `rhs` value if equal.
	///
	/// `self.last` must not be populated.
	fn cmp_next(&mut self, lhs: T, rhs: T) -> T {
		match lhs.cmp(&rhs) {
			// Note: Discard rhs
			Ordering::Equal => lhs,
			Ordering::Less => {
				self.last = Some(Either::Right(rhs));
				lhs
			},
			Ordering::Greater => {
				self.last = Some(Either::Left(lhs));
				rhs
			},
		}
	}
}

impl<T: Ord, Li: Iterator<Item = T>, Ri: Iterator<Item = T>> Iterator for DiscardingSortedMergeIter<T, Li, Ri> {
	type Item = T;

	fn next(&mut self) -> Option<Self::Item> {
		match self.last.take() {
			Some(Either::Left(lhs)) => match self.rhs.next() {
				Some(rhs) => Some(self.cmp_next(lhs, rhs)),
				None => Some(lhs),
			},
			Some(Either::Right(rhs)) => match self.lhs.next() {
				Some(lhs) => Some(self.cmp_next(lhs, rhs)),
				None => Some(rhs),
			},
			None => match (self.lhs.next(), self.rhs.next()) {
				(None, None) => None,
				(None, Some(func)) | (Some(func), None) => Some(func),
				(Some(lhs), Some(rhs)) => Some(self.cmp_next(lhs, rhs)),
			},
		}
	}
}
