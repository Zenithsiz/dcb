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
	#[allow(clippy::too_many_lines)] // This will be big, as it's the list of ALL known data
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
				desc:      "Start of the heap",
				start_pos: Pos(0x801ddf38),
			},
			Self::Bytes {
				name:      "something1_data1",
				desc:      "",
				start_pos: Pos(0x8006f984),
			},
			Self::Bytes {
				name:      "something1_data2",
				desc:      "",
				start_pos: Pos(0x80010000),
			},
			Self::Bytes {
				name:      "something5_data1",
				desc:      "",
				start_pos: Pos(0x8006fa20),
			},
			Self::Bytes {
				name:      "I_STAT_PTR",
				desc:      "",
				start_pos: Pos(0x80070aac),
			},
			Self::Bytes {
				name:      "I_MASK_PTR",
				desc:      "",
				start_pos: Pos(0x80070ab0),
			},
			Self::Bytes {
				name:      "DPCR_PTR",
				desc:      "",
				start_pos: Pos(0x80070ab4),
			},
			Self::Bytes {
				name:      "something5_data5",
				desc:      "",
				start_pos: Pos(0x8006fa5c),
			},
			Self::Bytes {
				name:      "FuncList1",
				desc:      "",
				start_pos: Pos(0x80070a88),
			},
			Self::Bytes {
				name:      "FuncList1Ptr",
				desc:      "Pointer to FuncList1",
				start_pos: Pos(0x80070aa8),
			},
			// Hardware registers
			// 0x1f80_1000 - 0x1f80_2fff
			Self::Bytes {
				name:      "I_STAT",
				desc:      "Interrupt status register",
				start_pos: Pos(0x1f801070),
			},
			Self::Bytes {
				name:      "I_MASK",
				desc:      "Interrupt mask register",
				start_pos: Pos(0x1f801074),
			},
			Self::Bytes {
				name:      "DPCR",
				desc:      "DMA Control register",
				start_pos: Pos(0x1f8010f0),
			},
			Self::Bytes {
				name:      "DICR",
				desc:      "DMA Interrupt register",
				start_pos: Pos(0x1f8010f4),
			},
			Self::Bytes {
				name:      "Timer0",
				desc:      "",
				start_pos: Pos(0x1f801100),
			},
			Self::Bytes {
				name:      "Timer1",
				desc:      "",
				start_pos: Pos(0x1f801110),
			},
			Self::Bytes {
				name:      "Timer2",
				desc:      "",
				start_pos: Pos(0x1f801120),
			},
		])
	}
}
