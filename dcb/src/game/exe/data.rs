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
pub struct Data<S: AsRef<str>> {
	/// Name
	pub name: S,

	/// Description
	pub desc: S,

	/// Start position
	pub start_pos: Pos,

	/// Data kind
	pub kind: DataKind,
}

impl<S: AsRef<str>> Data<S> {
	/// Returns the end position of this data
	pub fn end_pos(&self) -> Pos {
		self.start_pos + self.kind.size()
	}
}

/// Data kind
#[derive(Clone, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(derive_more::Display)]
pub enum DataKind {
	/// Ascii string
	// TODO: Maybe somehow get rid of the length?
	#[display(fmt = "str")]
	AsciiStr {
		/// String length
		len: u32,
	},

	/// Word
	#[display(fmt = "u32")]
	Word,

	/// Half-word
	#[display(fmt = "u16")]
	HalfWord,

	/// Byte
	#[display(fmt = "u8")]
	Byte,

	/// Array
	#[display(fmt = "[{ty}; {len}]")]
	Array {
		/// Array type
		ty: Box<DataKind>,

		/// Array length
		len: u32,
	},
}

impl DataKind {
	/// Returns the size of this data kind
	#[must_use]
	pub fn size(&self) -> u32 {
		match self {
			Self::AsciiStr { len } => len + 4 - (len % 4),
			Self::Word => 4,
			Self::HalfWord => 2,
			Self::Byte => 1,
			Self::Array { ty, len } => ty.size() * len,
		}
	}
}

impl<S: AsRef<str>> std::borrow::Borrow<Pos> for Data<S> {
	fn borrow(&self) -> &Pos {
		&self.start_pos
	}
}

impl<S: AsRef<str>> PartialEq for Data<S> {
	fn eq(&self, other: &Self) -> bool {
		// Only compare the start position
		self.start_pos.eq(&other.start_pos)
	}
}

impl<S: AsRef<str>> Eq for Data<S> {}

impl<S: AsRef<str>> std::hash::Hash for Data<S> {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.start_pos.hash(state);
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
		self.start_pos.cmp(&other.start_pos)
	}
}

impl Data<&'static str> {
	/// Returns an iterator of all known data
	#[allow(clippy::too_many_lines)] // This will be big, as it's the list of ALL known data
	pub fn known() -> impl Iterator<Item = Self> {
		std::array::IntoIter::new([
			Self {
				name:      "StackTop",
				desc:      "Stack top address",
				start_pos: Pos(0x8006dd44),
				kind:      DataKind::Word,
			},
			Self {
				name:      "StackSize",
				desc:      "Stack size",
				start_pos: Pos(0x8006dd48),
				kind:      DataKind::Word,
			},
			Self {
				name:      "ZeroStart",
				desc:      "Start of the zero section in `start`",
				start_pos: Pos(0x80077a08),
				kind:      DataKind::Word,
			},
			Self {
				name:      "HeapStart",
				desc:      "Start of the heap",
				start_pos: Pos(0x801ddf38),
				kind:      DataKind::Word,
			},
			Self {
				name:      "something1_data1",
				desc:      "",
				start_pos: Pos(0x8006f984),
				kind:      DataKind::Word,
			},
			Self {
				name:      "something1_data2",
				desc:      "",
				start_pos: Pos(0x80010000),
				kind:      DataKind::Word,
			},
			Self {
				name:      "something5_data1",
				desc:      "",
				start_pos: Pos(0x8006fa20),
				kind:      DataKind::HalfWord,
			},
			Self {
				name:      "I_STAT_PTR",
				desc:      "",
				start_pos: Pos(0x80070aac),
				kind:      DataKind::Word,
			},
			Self {
				name:      "I_MASK_PTR",
				desc:      "",
				start_pos: Pos(0x80070ab0),
				kind:      DataKind::Word,
			},
			Self {
				name:      "DPCR_PTR",
				desc:      "",
				start_pos: Pos(0x80070ab4),
				kind:      DataKind::Word,
			},
			Self {
				name:      "something5_data5",
				desc:      "",
				start_pos: Pos(0x8006fa5c),
				kind:      DataKind::Word,
			},
			Self {
				name:      "FuncList1",
				desc:      "",
				start_pos: Pos(0x80070a88),
				kind:      DataKind::Word,
			},
			Self {
				name:      "FuncList1Ptr",
				desc:      "Pointer to FuncList1",
				start_pos: Pos(0x80070aa8),
				kind:      DataKind::Word,
			},
			// Hardware registers
			// 0x1f80_1000 - 0x1f80_2fff
			Self {
				name:      "I_STAT",
				desc:      "Interrupt status register",
				start_pos: Pos(0x1f801070),
				kind:      DataKind::Word,
			},
			Self {
				name:      "I_MASK",
				desc:      "Interrupt mask register",
				start_pos: Pos(0x1f801074),
				kind:      DataKind::Word,
			},
			Self {
				name:      "DPCR",
				desc:      "DMA Control register",
				start_pos: Pos(0x1f8010f0),
				kind:      DataKind::Word,
			},
			Self {
				name:      "DICR",
				desc:      "DMA Interrupt register",
				start_pos: Pos(0x1f8010f4),
				kind:      DataKind::Word,
			},
			Self {
				name:      "Timer0",
				desc:      "",
				start_pos: Pos(0x1f801100),
				kind:      DataKind::Word,
			},
			Self {
				name:      "Timer1",
				desc:      "",
				start_pos: Pos(0x1f801110),
				kind:      DataKind::Word,
			},
			Self {
				name:      "Timer2",
				desc:      "",
				start_pos: Pos(0x1f801120),
				kind:      DataKind::Word,
			},
		])
	}
}
