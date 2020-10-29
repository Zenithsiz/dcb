//! Executable data

// Modules
pub mod all_data;

// Exports
pub use all_data::AllData;

// Imports
use crate::game::exe::Pos;

/// Executable data
#[derive(Clone, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum Data<S: AsRef<str>> {
	/// An ascii string
	Ascii {
		/// Name
		name: S,

		/// Description
		desc: S,

		/// Start position
		start_pos: Pos,
	},

	/// Bytes
	Bytes {
		/// Name
		name: S,

		/// Description
		desc: S,

		/// Start position
		start_pos: Pos,
	},
}

impl<S: AsRef<str>> std::borrow::Borrow<Pos> for Data<S> {
	fn borrow(&self) -> &Pos {
		match self {
			Self::Ascii { start_pos, .. } => start_pos,
			Self::Bytes { start_pos, .. } => start_pos,
		}
	}
}

impl<S: AsRef<str>> PartialEq for Data<S> {
	fn eq(&self, other: &Self) -> bool {
		// Only compare the start position
		self.start_pos().eq(&other.start_pos())
	}
}

impl<S: AsRef<str>> Eq for Data<S> {}

impl<S: AsRef<str>> std::hash::Hash for Data<S> {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.start_pos().hash(state);
	}
}

impl<S: AsRef<str>> PartialOrd for Data<S> {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		// Delegate to `eq` since we have a total order.
		Some(self.cmp(other))
	}
}
impl<S: AsRef<str>> Ord for Data<S> {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		// Only compare the start position
		self.start_pos().cmp(&other.start_pos())
	}
}

impl<S: AsRef<str>> Data<S> {
	/// Accesses the name of this data
	pub fn name(&self) -> &S {
		match self {
			Self::Ascii { name, .. } => name,
			Self::Bytes { name, .. } => name,
		}
	}

	/// Accesses the description of this data
	pub fn desc(&self) -> &S {
		match self {
			Self::Ascii { desc, .. } => desc,
			Self::Bytes { desc, .. } => desc,
		}
	}

	/// Accesses the start position of this data
	pub fn start_pos(&self) -> Pos {
		match self {
			Self::Ascii { start_pos, .. } => *start_pos,
			Self::Bytes { start_pos, .. } => *start_pos,
		}
	}
}


impl Data<&'static str> {
	/// Returns an iterator of all known data
	pub fn known() -> impl Iterator<Item = Self> {
		std::array::IntoIter::new([
			Self::Bytes {
				name:      "StackTop",
				desc:      "Stack top address",
				start_pos: Pos(0x8006dd44),
			},
			Self::Bytes {
				name:      "StackSize",
				desc:      "Stack size",
				start_pos: Pos(0x8006dd48),
			},
			Self::Bytes {
				name:      "ZeroStart",
				desc:      "Start of the zero section in `start`",
				start_pos: Pos(0x80077a08),
			},
			Self::Bytes {
				name:      "HeapStart",
				desc:      "start of the heap",
				start_pos: Pos(0x801ddf38),
			},
		])
	}
}
