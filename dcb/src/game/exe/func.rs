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
use indoc::indoc;
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

	/// Labels
	pub labels: HashMap<Pos, S>,

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
	#[allow(clippy::too_many_lines)] // This will be big, as it's the list of ALL known functions
	pub fn known() -> impl Iterator<Item = Self> {
		std::array::IntoIter::new([
			Self {
				name:      "InitHeap",
				signature: "fn(addr: *int, size: int)",
				desc:      "Calls A(0x39)",
				comments:  hashmap! {
					Pos(0x8006a738) => "Register tailcall. Likely to prevent calling in KSEG0 and do it in KUSEG",
					Pos(0x8006a73c) => "arg: 0x39",
				},
				labels:    hashmap! {},
				start_pos: Pos(0x8006a734),
				end_pos:   Pos(0x8006a744),
			},
			Self {
				name:      "start",
				signature: "fn()",
				desc:      "Executable start",
				comments:  hashmap! {
					Pos(0x80056280) => "Zero out ZeroStart .. HeapStart word by word.",
					Pos(0x80056284) => "^",
					Pos(0x80056288) => "^",
					Pos(0x8005628c) => "^",
					Pos(0x800562a8) => "Initialize stack to (*StackTop - 0x10) | 0x80000000",
					Pos(0x800562f8) => "args: (HeapStart, (*StackTop - 0x10) - *StackSize - (HeapStart & 0x1fff_ffff))",
					Pos(0x8005630c) => "args: (HeapStart + 0x4, ...?)",
					Pos(0x80056324) => "args: (something1_data2, something1_data2)",
				},
				labels:    hashmap! {
					Pos(0x80056280) => "zero_loop",
				},
				start_pos: Pos(0x80056270),
				end_pos:   Pos(0x80056330),
			},
			Self {
				name:      "something1",
				signature: "fn(arg: int)",
				desc:      indoc! {"
					This function checks if *something1_data1 is positive, if so decreases
					it by 1 and calls call_func_arr with (something1_data2, something1_data2).
				"},
				comments:  hashmap! {
					Pos(0x80056348) => "If *something1_data1 == 0, skip",
					Pos(0x8005634c) => "Else decrease it by 1 and save it.",
					Pos(0x80056368) => "Then call call_func_arr with args (something1_data2, something1_data2)",
				},
				labels:    hashmap! {
					Pos(0x80056370) => "skip",
				},
				start_pos: Pos(0x80056330),
				end_pos:   Pos(0x80056388),
			},
			Self {
				name:      "call_func_arr",
				signature: "fn(start: fn(), end: fn())",
				desc:      "",
				comments:  hashmap! {
					Pos(0x800563a0) => "if `start >= end`, skip",
					Pos(0x800563b0) => "If *start == 0, skip call",
					Pos(0x800563b8) => "Else call *start",
					Pos(0x800563c0) => "start++",
					Pos(0x800563c8) => "If `start < end`, restart",
				},
				labels:    hashmap! {
					Pos(0x800563a8) => "loop",
					Pos(0x800563c0) => "skip_call",
					Pos(0x800563d0) => "exit",
				},
				start_pos: Pos(0x80056388),
				end_pos:   Pos(0x800563e4),
			},
			Self {
				name:      "something2",
				signature: "fn(start: *int)",
				desc:      "",
				comments:  hashmap! {
					Pos(0x80013e54) => "args: (start)",
					Pos(0x80013e6c) => "args: (0)",
				},
				labels:    hashmap! {
					Pos(0x80013ef4) => "0",
					Pos(0x80013f48) => "1",
					Pos(0x80013f54) => "2",
					Pos(0x80013f6c) => "3",
					Pos(0x80013f8c) => "4",
				},
				start_pos: Pos(0x80013e4c),
				end_pos:   Pos(0x80013fa4),
			},
			Self {
				name:      "something3",
				signature: "fn()",
				desc:      "",
				comments:  hashmap! {
					Pos(0x80056604) => "Loads FuncList1[3]",
					Pos(0x8005660c) => "Calls FuncList1[3] (i.e. something5)",
				},
				labels:    hashmap! {},
				start_pos: Pos(0x800565f4),
				end_pos:   Pos(0x80056624),
			},
			Self {
				name:      "something4",
				signature: "fn()",
				desc:      "",
				comments:  hashmap! {},
				labels:    hashmap! {
					Pos(0x80056ac0) => "0",
					Pos(0x80056ae0) => "1",
					Pos(0x80056b04) => "2",
					Pos(0x80056b1c) => "3",
					Pos(0x80056b34) => "4",
					Pos(0x80056b44) => "5",
					Pos(0x80056b54) => "6",
					Pos(0x80056b58) => "7",
				},
				start_pos: Pos(0x80056a30),
				end_pos:   Pos(0x80056b78),
			},
			Self {
				name:      "something5",
				signature: "fn()",
				desc:      "",
				comments:  hashmap! {
					Pos(0x8005679c) => "Loads *(short*)something5_data1",
					Pos(0x800567a4) => "If the loaded value is not zero, exit",
					Pos(0x800567c0) => "Zero out the top half of `I_MASK_PTR`, which seems to be garbage",
					Pos(0x800567c4) => "Then read the top half of `I_MASK_PTR` and zero-extend it, which is still garbage?",

					Pos(0x800567dc) => "Set the DMA control registers to 0x3333_3333",
					Pos(0x800567e0) => "args: (something5_data1, 0x3333_3333)",

					Pos(0x800567e8) => "Save all registers with `save_registers` and check return value",
					Pos(0x800567f0) => "If the return value isn't 0, call `func_831`. This shouldn't happen, as `save_registers` always returns 0",
				},
				labels:    hashmap! {
					Pos(0x80056800) => "skip_call",
					Pos(0x80056850) => "exit",
				},
				start_pos: Pos(0x80056788),
				end_pos:   Pos(0x80056860),
			},
			Self {
				name:      "save_registers",
				signature: "fn(int* pos)",
				desc:      indoc! {"
					Saves the following registers in `pos[0x0 .. 0x30]`.
					$ra, $gp, $sp, $fp,
					$s0, $s1, $s2, $s3,
					$s4, $s5, $s6, $s7,
				"},
				comments:  hashmap! {},
				labels:    hashmap! {},
				start_pos: Pos(0x8006a674),
				end_pos:   Pos(0x8006a6b0),
			},
			Self {
				name:      "memset_zero",
				signature: "fn(int* ptr, int size)",
				desc:      indoc! {"
					Zeroes out the memory at `ptr` for `size` words.
				"},
				comments:  hashmap! {
					Pos(0x80056c90) => "If size == 0, return",
					Pos(0x80056c94) => "size--",
					Pos(0x80056c9c) => "*ptr = 0",
					Pos(0x80056ca0) => "size--",
					Pos(0x80056ca4) => "While size != -1, continue",
					Pos(0x80056ca8) => "ptr++"
				},
				labels:    hashmap! {
					Pos(0x80056c9c) => "loop",
					Pos(0x80056cac) => "exit",
				},
				start_pos: Pos(0x80056c90),
				end_pos:   Pos(0x80056cb4),
			},
		])
	}
}
