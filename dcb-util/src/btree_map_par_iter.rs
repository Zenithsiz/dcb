//! [`BTreeMap`] parallel iterator

// Imports
use either::Either;
use std::collections::{btree_map, BTreeMap};


/// Iterator over two `BTreeMap`, proving either one or both for each key, in ascending order
// TODO: Generalize this to any two iterators?
#[derive(Clone, Debug)]
pub struct BTreeMapParIter<'a, K, VL, VR> {
	/// Left map
	left: btree_map::Iter<'a, K, VL>,

	/// Right map
	right: btree_map::Iter<'a, K, VR>,

	/// Currently cached value
	value: Option<(&'a K, Either<&'a VL, &'a VR>)>,
}

impl<'a, K, VL, VR> BTreeMapParIter<'a, K, VL, VR> {
	/// Creates a new iterator from two trees
	#[must_use]
	pub fn new(left: &'a BTreeMap<K, VL>, right: &'a BTreeMap<K, VR>) -> Self {
		Self {
			left:  left.iter(),
			right: right.iter(),
			value: None,
		}
	}

	/// Returns the next left value
	pub fn next_left(&mut self) -> Option<(&'a K, &'a VL)> {
		// Check if we have it cached
		if let Some((key, value)) = self.value.take() {
			match value {
				// If we did, return it
				Either::Left(value) => return Some((key, value)),
				// If it wasn't on the left, put it back
				Either::Right(_) => self.value = Some((key, value)),
			}
		}

		// If we didn't have it cached get it from the iterator
		self.left.next()
	}

	/// Returns the next right value
	pub fn next_right(&mut self) -> Option<(&'a K, &'a VR)> {
		// Check if we have it cached
		if let Some((key, value)) = self.value.take() {
			match value {
				// If we did, return it
				Either::Right(value) => return Some((key, value)),
				// If it wasn't on the right, put it back
				Either::Left(_) => self.value = Some((key, value)),
			}
		}

		// If we didn't have it cached get it from the iterator
		self.right.next()
	}
}

impl<'a, K, VL, VR> Iterator for BTreeMapParIter<'a, K, VL, VR>
where
	K: Ord,
{
	type Item = (&'a K, ParIterValue<'a, VL, VR>);

	fn next(&mut self) -> Option<Self::Item> {
		match (self.next_left(), self.next_right()) {
			// If we only got one of each value, just return it
			(Some((key, left)), None) => Some((key, ParIterValue::Left(left))),
			(None, Some((key, right))) => Some((key, ParIterValue::Right(right))),

			// If we got both with equal keys, return them both
			(Some((left_key, left)), Some((right_key, right))) if left_key == right_key => {
				Some((left_key, ParIterValue::Both(left, right)))
			},

			// If we got both, but without equal keys, emit the first and store the other.
			// Note: In all of these branches, `self.value` is empty, as we call both `self.next_{left, right}`
			//       functions.
			(Some((left_key, left)), Some((right_key, right))) => match left_key < right_key {
				true => {
					self.value = Some((right_key, Either::Right(right)));
					Some((left_key, ParIterValue::Left(left)))
				},
				false => {
					self.value = Some((left_key, Either::Left(left)));
					Some((right_key, ParIterValue::Right(right)))
				},
			},

			// Else we got none
			(None, None) => None,
		}
	}
}

/// Iterator value
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum ParIterValue<'a, VL, VR> {
	/// Only left
	Left(&'a VL),

	/// Only right
	Right(&'a VR),

	/// Both
	Both(&'a VL, &'a VR),
}

impl<'a, VL, VR> ParIterValue<'a, VL, VR> {
	/// Returns a pair of options describing this value
	#[must_use]
	pub const fn into_opt_pair(self) -> (Option<&'a VL>, Option<&'a VR>) {
		match self {
			Self::Left(left) => (Some(left), None),
			Self::Right(right) => (None, Some(right)),
			Self::Both(left, right) => (Some(left), Some(right)),
		}
	}
}
