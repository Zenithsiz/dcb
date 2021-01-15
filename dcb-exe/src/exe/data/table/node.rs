//! Data table nodes

// Imports
use crate::{exe::Data, Pos};
use std::{
	borrow::Borrow,
	cmp::Ordering,
	collections::BTreeSet,
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

/// Error for [`DataNode::insert`]
#[derive(Debug, thiserror::Error)]
pub enum InsertError {
	/// The data location is not part of this node
	#[error("The data location {_0} is not part of this node")]
	NotContained(Data),

	/// The data location overlapped
	#[error("The data locations {_0} and {_1} overlap")]
	Intersection(Data, Data),

	/// Unable to insert into child node
	#[error("Unable to insert into child {_0}")]
	InsertChild(Data, #[source] Box<Self>),
}

impl DataNode {
	/// Creates a new data node
	pub const fn new(data: Data) -> Self {
		Self {
			data,
			nodes: BTreeSet::new(),
		}
	}

	/// Returns the data in this node
	pub const fn data(&self) -> &Data {
		&self.data
	}

	/// Inserts a new data into this node
	///
	/// If the data already existed with the same position and type, it will
	/// not be inserted, instead it will be returned.
	// TODO: Get rid of all these clones.
	pub fn try_insert(&mut self, data: Data) -> Result<Option<Data>, InsertError> {
		// If the data isn't contained in ourselves, return Err
		if !self.contains(&data) {
			return Err(InsertError::NotContained(data));
		}

		// Check the first node behind it to insert
		if let Some(node) = self.nodes.range(..=data.start_pos()).next_back() {
			// If it's equal to the node, don't replace it
			if node.data.start_pos() == data.start_pos() && node.data.ty() == data.ty() {
				return Ok(Some(data));
			}
			// If it contains it, insert it there
			else if node.contains(&data) {
				let node_data = node.data.clone();
				let node_pos = node.data.start_pos();
				return self::btree_set_modify(&mut self.nodes, &node_pos, |node| node.try_insert(data))
					.map_err(move |err| InsertError::InsertChild(node_data, Box::new(err)));
			}
			// If it doesn't contain it, but intersects, return Err
			else if node.intersects(&data) {
				return Err(InsertError::Intersection(node.data.clone(), data));
			}
		}

		// Else make sure it doesn't intersect the node after
		if let Some(node) = self.get_next_from(data.start_pos()) {
			if node.intersects(&data) {
				return Err(InsertError::Intersection(node.data.clone(), data));
			}
		}

		// And insert it
		// TODO: Check bug here where this can fail
		assert!(self.nodes.insert(Self::new(data)), "No node with this position should exist");
		Ok(None)
	}

	/// Checks if a data is contained in this node
	pub fn contains(&self, other: &Data) -> bool {
		self::range_contains_range(self.data.pos_range(), other.pos_range())
	}

	/// Checks if a data intersects this node
	pub fn intersects(&self, other: &Data) -> bool {
		self::range_intersect(self.data.pos_range(), other.pos_range())
	}

	/// Returns a data node containing `pos`
	pub fn get_containing(&self, pos: Pos) -> Option<&Self> {
		// Note: We search backwards as the nodes will be sorted
		//       by their start position
		self.nodes.range(..=pos).next_back().filter(|node| node.data.contains_pos(pos))
	}

	/// Returns the deepest data node containing `pos`
	pub fn get_containing_deepest(&self, pos: Pos) -> Option<&Self> {
		// Go as far down the tree as we can
		let mut cur_node = self.get_containing(pos)?;
		while let Some(node) = cur_node.get_containing(pos) {
			cur_node = node;
		}
		Some(cur_node)
	}

	/// Returns the first data node after `pos`
	pub fn get_next_from(&self, pos: Pos) -> Option<&Self> {
		self.nodes.range((Bound::Excluded(pos), Bound::Unbounded)).next()
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

/// Checks if a range contains another range
fn range_contains_range<T: PartialOrd>(bigger: Range<T>, smaller: Range<T>) -> bool {
	smaller.start >= bigger.start && smaller.end <= bigger.end
}

/// Checks if two ranges are disjoint
fn range_disjoint<T: PartialOrd>(lhs: Range<T>, rhs: Range<T>) -> bool {
	lhs.start <= rhs.end || rhs.start <= lhs.end
}

/// Checks if two ranges intersect
fn range_intersect<T: PartialOrd>(lhs: Range<T>, rhs: Range<T>) -> bool {
	!self::range_disjoint(lhs, rhs)
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
