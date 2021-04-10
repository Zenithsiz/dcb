//! Errors

// Imports
use crate::Data;

/// Error for [`DataNode::insert`](super::DataNode::insert)
#[derive(Debug, thiserror::Error)]
pub enum InsertError {
	/// The data location is not part of this node
	#[error("The data location {_0} is not part of this node")]
	NotContained(Data),

	/// The data location overlapped
	#[error("The data locations {data} and {intersecting} intersect")]
	Intersection {
		/// The data being inserted
		data: Data,

		/// The data that it intersects with
		intersecting: Data,
	},

	/// The data location already existed
	#[error("The data locations {data} and {duplicate} are duplicates")]
	Duplicate {
		/// The data being inserted
		data: Data,

		/// The data that it is a duplicate of
		duplicate: Data,
	},

	/// Attempted to insert heuristic data into known non-marker data
	#[error("The heuristics data location {data} cannot be inserted into known non-marker data location {known}")]
	InsertHeuristicsIntoNonMarkerKnown {
		/// The data being inserted
		data: Data,

		/// The known non-marker data
		known: Data,
	},

	/// Unable to insert into child node
	#[error("Unable to insert into child {child}")]
	InsertChild {
		/// The child being inserted into
		child: Data,

		/// Underlying error
		#[source]
		err: Box<Self>,
	},
}

impl InsertError {
	/// Returns the data being inserted
	#[must_use]
	pub fn data(&self) -> &Data {
		match self {
			Self::NotContained(data) |
			Self::Intersection { data, .. } |
			Self::Duplicate { data, .. } |
			Self::InsertHeuristicsIntoNonMarkerKnown { data, .. } => data,
			Self::InsertChild { err, .. } => err.data(),
		}
	}
}
