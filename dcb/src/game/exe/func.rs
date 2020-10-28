//! Executable functions

// Modules
pub mod funcs;
pub mod iter;

// Exports
pub use funcs::Funcs;
pub use iter::WithInstructionsIter;
use maplit::hashmap;

// Imports
use crate::game::exe::Pos;
use std::collections::HashMap;

/// A function within the executable
#[derive(Clone, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Func<S: AsRef<str>> {
	/// Function name
	pub name: S,

	/// Function signature
	pub signature: S,

	/// Description
	pub desc: S,

	/// Comments
	pub comments: HashMap<Pos, S>,

	/// Start position
	pub start_pos: Pos,

	/// End position (non-inclusive)
	pub end_pos: Pos,
}

impl<S: AsRef<str>> PartialEq for Func<S> {
	fn eq(&self, other: &Self) -> bool {
		// Only compare the start position
		self.start_pos.eq(&other.start_pos)
	}
}

impl<S: AsRef<str>> Eq for Func<S> {}

impl<S: AsRef<str>> PartialOrd for Func<S> {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		// Delegate to `eq` since we have a total order.
		Some(self.cmp(other))
	}
}
impl<S: AsRef<str>> Ord for Func<S> {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		// Only compare the start position
		self.start_pos.cmp(&other.start_pos)
	}
}

impl Func<&'static str> {
	/// Returns an iterator of all known functions
	pub fn known() -> impl Iterator<Item = Self> {
		std::array::IntoIter::new([
			Self {
				name:      "InitHeap",
				signature: "void(int* addr, unsigned int size)",
				desc:      "Calls A(0x39)",
				comments:  hashmap! {},
				start_pos: Pos(0x8006a734),
				end_pos:   Pos(0x8006a744),
			},
			Self {
				name:      "start",
				signature: "void(void)",
				desc:      "Executable start",
				comments:  hashmap! {
					Pos(0x80056280) => "Zero out 0x80077a08 .. 0x801ddf38 word by word.",
					Pos(0x80056284) => "^",
					Pos(0x80056288) => "^",
					Pos(0x8005628c) => "^",
					Pos(0x800562f8) => "InitHeap(0x8007f988, ???)",
					Pos(0x8005630c) => "func_1025(0x8007f98c)",
					Pos(0x80056324) => "func_1026(string_0, string_0)",
				},
				start_pos: Pos(0x80056270),
				end_pos:   Pos(0x80056330),
			},
			Self {
				name:      "func_1025",
				signature: "void(int*)",
				desc:      "",
				comments:  hashmap! {
					Pos(0x80013ef4) => "Called indefinitely?",
					Pos(0x80013efc) => "^ Due to this loop"
				},
				start_pos: Pos(0x80013e4c),
				end_pos:   Pos(0x80013f04),
			},
			Self {
				name:      "func_446",
				signature: "int(int)",
				desc:      "",
				comments:  hashmap! {},
				start_pos: Pos(0x80069124),
				end_pos:   Pos(0x80069150),
			},
		])
	}
}
