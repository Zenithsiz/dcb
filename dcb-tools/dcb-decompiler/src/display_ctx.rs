//! Display context

// Imports
use dcb_exe::{inst, ExeReader, Func, Pos};
use std::fmt;

/// Displaying context for instructions.
pub struct DisplayCtx<'a> {
	/// Executable
	exe: &'a ExeReader,

	/// Current function, if any.
	func: Option<&'a Func>,

	/// Current Position
	pos: Pos,
}

impl<'a> DisplayCtx<'a> {
	/// Creates a new display context
	pub fn new(exe: &'a ExeReader, func: Option<&'a Func>, pos: Pos) -> Self {
		Self { exe, func, pos }
	}
}

/// Label display for `DisplayCtx::pos_label`
pub enum LabelDisplay<'a> {
	/// Label within the current function
	CurFuncLabel { label: &'a str },

	/// Label within another function
	OtherFuncLabel { func: &'a str, label: &'a str },

	/// A function itself
	OtherFunc { func: &'a str },

	/// A data
	Data { name: &'a str },
}

impl<'a> fmt::Display for LabelDisplay<'a> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			LabelDisplay::CurFuncLabel { label } => write!(f, ".{label}"),
			LabelDisplay::OtherFuncLabel { func, label } => write!(f, "{func}.{label}"),
			LabelDisplay::OtherFunc { func } => write!(f, "{func}"),
			LabelDisplay::Data { name } => write!(f, "{name}"),
		}
	}
}

impl<'a> inst::DisplayCtx for DisplayCtx<'a> {
	type Label = LabelDisplay<'a>;

	fn cur_pos(&self) -> Pos {
		self.pos
	}

	fn pos_label(&self, pos: Pos) -> Option<(Self::Label, i64)> {
		// Try to check if `pos` is a label within the current function
		if let Some(label) = self.func.and_then(|func| func.labels.get(&pos)) {
			return Some((LabelDisplay::CurFuncLabel { label }, 0));
		}

		// Else check if there's any function containing this position
		if let Some(func) = self.exe.func_table().get_containing(pos) {
			// If so, check if any of the labels match it
			if let Some(label) = func.labels.get(&pos) {
				return Some((
					LabelDisplay::OtherFuncLabel {
						func: &func.name,
						label,
					},
					0,
				));
			}

			// If no label matches and this is a known function and not the function itself, warn
			if pos != func.start_pos && !func.kind.is_heuristics() {
				log::warn!(
					"Display context was queried for {}, which resides within {}, but no label was found with the \
					 position",
					pos,
					func.name
				);
			}

			// Then just return the function with an offset.
			return Some((LabelDisplay::OtherFunc { func: &func.name }, pos - func.start_pos));
		}

		// Else check if any data contains it.
		if let Some(data) = self.exe.data_table().get_containing(pos) {
			return Some((LabelDisplay::Data { name: data.name() }, pos - data.start_pos()));
		}

		// Else we don't have any label for it.
		None
	}
}
