//! Instructions with a target

// Imports
use crate::Pos;

/// Instructions that have a target
pub trait InstTarget {
	/// Returns this instruction's target
	fn target(&self, pos: Pos) -> Pos;
}
