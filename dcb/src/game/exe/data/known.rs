//! Known data locations
//!
//! This module stores the [`Data::known`] function
//! that returns all known data locations.
//!
//! It is a separate module, as the known data locations
//! occupy a large amount of space.

// Imports
use super::{Data, DataKind, Pos};

impl Data<&'static str> {
	/// Returns an iterator of all known data
	#[allow(clippy::too_many_lines)] // This will be big, as it's the list of ALL known data
	pub fn known() -> impl Iterator<Item = Self> {
		std::array::IntoIter::new([
			Self {
				name: "StackTop",
				desc: "Stack top address",
				pos:  Pos(0x8006dd44),
				kind: DataKind::Word,
			},
			Self {
				name: "StackSize",
				desc: "Stack size",
				pos:  Pos(0x8006dd48),
				kind: DataKind::Word,
			},
			Self {
				name: "ZeroStart",
				desc: "Start of the zero section in `start`",
				pos:  Pos(0x80077a08),
				kind: DataKind::Word,
			},
			Self {
				name: "HeapStart",
				desc: "Start of the heap",
				pos:  Pos(0x801ddf38),
				kind: DataKind::Word,
			},
			Self {
				name: "something1_data1",
				desc: "",
				pos:  Pos(0x8006f984),
				kind: DataKind::Word,
			},
			Self {
				name: "something1_data2",
				desc: "",
				pos:  Pos(0x80010000),
				kind: DataKind::Word,
			},
			Self {
				name: "something5_data1",
				desc: "",
				pos:  Pos(0x8006fa20),
				kind: DataKind::HalfWord,
			},
			Self {
				name: "I_STAT_PTR",
				desc: "",
				pos:  Pos(0x80070aac),
				kind: DataKind::Word,
			},
			Self {
				name: "I_MASK_PTR",
				desc: "",
				pos:  Pos(0x80070ab0),
				kind: DataKind::Word,
			},
			Self {
				name: "DPCR_PTR",
				desc: "",
				pos:  Pos(0x80070ab4),
				kind: DataKind::Word,
			},
			Self {
				name: "something5_data5",
				desc: "",
				pos:  Pos(0x8006fa5c),
				kind: DataKind::Word,
			},
			Self {
				name: "FuncList1",
				desc: "",
				pos:  Pos(0x80070a88),
				kind: DataKind::Word,
			},
			Self {
				name: "FuncList1Ptr",
				desc: "Pointer to FuncList1",
				pos:  Pos(0x80070aa8),
				kind: DataKind::Word,
			},
			// Hardware registers
			// 0x1f80_1000 - 0x1f80_2fff
			Self {
				name: "I_STAT",
				desc: "Interrupt status register",
				pos:  Pos(0x1f801070),
				kind: DataKind::Word,
			},
			Self {
				name: "I_MASK",
				desc: "Interrupt mask register",
				pos:  Pos(0x1f801074),
				kind: DataKind::Word,
			},
			Self {
				name: "DPCR",
				desc: "DMA Control register",
				pos:  Pos(0x1f8010f0),
				kind: DataKind::Word,
			},
			Self {
				name: "DICR",
				desc: "DMA Interrupt register",
				pos:  Pos(0x1f8010f4),
				kind: DataKind::Word,
			},
			Self {
				name: "Timer0",
				desc: "",
				pos:  Pos(0x1f801100),
				kind: DataKind::Word,
			},
			Self {
				name: "Timer1",
				desc: "",
				pos:  Pos(0x1f801110),
				kind: DataKind::Word,
			},
			Self {
				name: "Timer2",
				desc: "",
				pos:  Pos(0x1f801120),
				kind: DataKind::Word,
			},
		])
	}
}
