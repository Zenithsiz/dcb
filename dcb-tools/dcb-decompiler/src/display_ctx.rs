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
	CurFuncLabel(&'a str),

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
			LabelDisplay::CurFuncLabel(label) => write!(f, ".{label}"),
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
		// Try to get a label for the current function, if it exists
		if let Some(label) = self.func.and_then(|func| func.labels.get(&pos)) {
			return Some((LabelDisplay::CurFuncLabel(label), 0));
		}

		// Try to get a function from it
		if let Some(func) = self.exe.func_table().get_containing(pos) {
			// And then one of it's labels
			if let Some(label) = func.labels.get(&pos) {
				return Some((
					LabelDisplay::OtherFuncLabel {
						func: &func.name,
						label,
					},
					0,
				));
			}

			// Else just any position in it
			return Some((LabelDisplay::OtherFunc { func: &func.name }, pos - func.start_pos));
		}

		// Else try a data
		if let Some(data) = self.exe.data_table().get_containing(pos) {
			return Some((LabelDisplay::Data { name: data.name() }, pos - data.start_pos()));
		}

		None
	}
}
