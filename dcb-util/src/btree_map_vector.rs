//! `BTreeMap<K, Vec<V>>`.

// Imports
use std::{borrow::Borrow, collections::BTreeMap, ops::RangeBounds};


/// A b-tree map with `Vec<V>` values, sorted by
/// insertion order.
#[derive(PartialEq, Clone, Debug)]
pub struct BTreeMapVector<K, V> {
	/// The underlying map
	map: BTreeMap<K, Vec<V>>,
}

impl<K, V> BTreeMapVector<K, V> {
	/// Creates a new, empty map.
	#[must_use]
	pub fn new() -> Self
	where
		K: Ord,
	{
		Self { map: BTreeMap::new() }
	}

	/// Returns a range of this map
	pub fn range<T, R>(&self, range: R) -> impl DoubleEndedIterator<Item = (&K, &V)>
	where
		T: Ord + ?Sized,
		R: RangeBounds<T>,
		K: Borrow<T> + Ord,
	{
		self.map
			.range(range)
			.flat_map(|(k, values)| values.iter().map(move |v| (k, v)))
	}

	/// Inserts a key-value pair into the map
	pub fn insert(&mut self, key: K, value: V)
	where
		K: Ord,
	{
		let values = self.map.entry(key).or_default();
		values.push(value);
	}
}

impl<K: Ord, V> Default for BTreeMapVector<K, V> {
	fn default() -> Self {
		Self::new()
	}
}
