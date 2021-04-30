//! Errors

// Imports
use crate::Pos;

/// Validation error
#[derive(Debug, thiserror::Error)]
pub enum ValidateError<'a> {
	/// Function range is invalid
	#[error("Range {start_pos}..{end_pos} is invalid")]
	InvalidRange {
		/// Start position
		start_pos: Pos,

		/// End position
		end_pos: Pos,
	},

	/// Label position is outside of function
	#[error("Label {label} @ {pos} is outside of function bounds")]
	LabelPosOutOfBounds {
		/// Position
		pos: Pos,

		/// Label
		label: &'a str,
	},

	/// Comment position is outside of function
	#[error("Comment {comment} @ {pos} is outside of function bounds")]
	CommentPosOutOfBounds {
		/// Position
		pos: Pos,

		/// Comment
		comment: &'a str,
	},
}
