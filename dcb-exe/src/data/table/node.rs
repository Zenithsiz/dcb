//! Data table nodes

// Modules
pub mod error;

// Exports
pub use error::InsertError;

// Imports
use crate::{Data, DataType, Pos};
use std::{
	borrow::Borrow,
	cmp::{self, Ordering},
	collections::BTreeSet,
	fmt,
	ops::{Bound, Range},
};

/// A data node
///
/// Represents a level of data, with possible children
#[derive(Clone, Debug)]
pub struct DataNode {
	/// The data in this node
	data: Data,

	/// All children
	nodes: BTreeSet<Self>,
}

impl DataNode {
	/// Creates a new data node
	#[must_use]
	pub const fn new(data: Data) -> Self {
		Self {
			data,
			nodes: BTreeSet::new(),
		}
	}

	/// Returns the data in this node
	#[must_use]
	pub const fn data(&self) -> &Data {
		&self.data
	}

	/// Returns all nodes in this node
	pub fn nodes(&self) -> impl Iterator<Item = &Self> {
		self.nodes.iter()
	}

	/// Inserts a new data into this node
	// TODO: Get rid of all these clones.
	pub fn insert(&mut self, data: Data) -> Result<(), InsertError> {
		// If the data isn't contained in ourselves, return Err
		if !self.contains(&data) {
			return Err(InsertError::NotContained(data));
		}

		// Check the first node behind it to insert
		if let Some(node) = self.nodes.range(..=data.start_pos()).next_back() {
			// If it's position range is the same, ignore it
			if node.data.start_pos() == data.start_pos() && node.data.ty() == data.ty() {
				return Err(InsertError::Duplicate {
					data,
					duplicate: node.data.clone(),
				});
			}
			// If it contains it, check if we can insert it there
			else if node.contains(&data) {
				// If `data` is heuristics and `node`'s data is known and not a marker, return Err
				if data.kind().is_heuristics() && node.data.kind().is_known() && !matches!(node.data.ty(), DataType::Marker { .. }) {
					return Err(InsertError::InsertHeuristicsIntoNonMarkerKnown {
						data,
						known: node.data.clone(),
					});
				}

				// Else try to insert it
				let node_pos = node.data.start_pos();
				return self::btree_set_modify(&mut self.nodes, &node_pos, |node| {
					node.insert(data).map_err(|err| InsertError::InsertChild {
						child: node.data.clone(),
						err:   Box::new(err),
					})
				});
			}
			// If it doesn't contain it, but intersects, return Err
			else if node.intersects(&data) {
				return Err(InsertError::Intersection {
					data,
					intersecting: node.data.clone(),
				});
			};
		}

		// Else make sure it doesn't intersect the node after
		if let Some(node) = self.get_next_from(data.start_pos()) {
			if node.intersects(&data) {
				return Err(InsertError::Intersection {
					data,
					intersecting: node.data.clone(),
				});
			}
		}

		// And insert it
		assert_eq!(self.nodes.replace(Self::new(data)), None, "No node with this position should exist",);
		Ok(())
	}

	/// Checks if a data is contained in this node
	#[must_use]
	pub fn contains(&self, other: &Data) -> bool {
		self::range_contains_range(self.data.pos_range(), other.pos_range())
	}

	/// Checks if a data intersects this node
	#[must_use]
	pub fn intersects(&self, other: &Data) -> bool {
		self::range_intersect(self.data.pos_range(), other.pos_range())
	}

	/// Returns a data node containing `pos`
	#[must_use]
	pub fn get_containing(&self, pos: Pos) -> Option<&Self> {
		// Note: We search backwards as the nodes will be sorted
		//       by their start position
		self.nodes.range(..=pos).next_back().filter(|node| node.data.contains_pos(pos))
	}

	/// Returns the deepest data node containing `pos`
	#[must_use]
	pub fn get_containing_deepest(&self, pos: Pos) -> Option<&Self> {
		// Go as far down the tree as we can
		let mut cur_node = self.get_containing(pos)?;
		while let Some(node) = cur_node.get_containing(pos) {
			cur_node = node;
		}
		Some(cur_node)
	}

	/// Returns the first data node after `pos`
	#[must_use]
	pub fn get_next_from(&self, pos: Pos) -> Option<&Self> {
		self.nodes.range((Bound::Excluded(pos), Bound::Unbounded)).next()
	}

	/// Formats this node with a depth
	fn fmt_with_depth(&self, depth: usize, f: &mut fmt::Formatter) -> fmt::Result {
		for _ in 0..depth {
			write!(f, "\t")?;
		}
		writeln!(f, "{}", self.data)?;
		for node in &self.nodes {
			node.fmt_with_depth(depth + 1, f)?;
		}
		Ok(())
	}
}

/// Borrows the start position of the node
impl Borrow<Pos> for DataNode {
	fn borrow(&self) -> &Pos {
		self.data.start_pos_ref()
	}
}

impl PartialEq for DataNode {
	fn eq(&self, other: &Self) -> bool {
		self.cmp(other).is_eq()
	}
}

impl Eq for DataNode {}

impl PartialOrd for DataNode {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for DataNode {
	fn cmp(&self, other: &Self) -> Ordering {
		// If `other` is contained in `self` or backwards, we're equal
		if self.contains(&other.data) || other.contains(&self.data) {
			return Ordering::Equal;
		}

		// Else make sure that we're both disjoint
		assert!(
			self::range_disjoint(self.data.pos_range(), other.data.pos_range()),
			"Cannot compare overlapping nodes"
		);

		// Else just check which one is in front
		match self.data.pos_range().start > other.data.pos_range().start {
			true => Ordering::Greater,
			false => Ordering::Less,
		}
	}
}

impl fmt::Display for DataNode {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.fmt_with_depth(0, f)
	}
}

/// Checks if a range contains another range
fn range_contains_range<T: Ord>(bigger: Range<T>, smaller: Range<T>) -> bool {
	smaller.start >= bigger.start && smaller.end <= bigger.end
}

/// Checks if two ranges are disjoint
fn range_disjoint<T: Ord>(lhs: Range<T>, rhs: Range<T>) -> bool {
	!self::range_intersect(lhs, rhs)
}

/// Checks if two ranges intersect
fn range_intersect<T: Ord>(lhs: Range<T>, rhs: Range<T>) -> bool {
	cmp::max(lhs.start, rhs.start) < cmp::min(lhs.end, rhs.end)
}

/// Removes, modifies and re-inserts a value back into a set
///
/// Panics if `element` doesn't exist.
fn btree_set_modify<T: Ord + Borrow<Q> + std::fmt::Debug, Q: Ord, U>(set: &mut BTreeSet<T>, element: &Q, f: impl FnOnce(&mut T) -> U) -> U {
	let mut node = set.take(element).expect("Element didn't exist");
	let res = f(&mut node);
	set.replace(node).expect_none("Just removed it");
	res
}
