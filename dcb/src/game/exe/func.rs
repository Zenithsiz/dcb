//! Executable functions

// Imports
use crate::game::exe::Pos;
use indoc::indoc;
use std::borrow::Cow;

/// A function within the executable
#[derive(PartialEq, Eq, Clone, Hash, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Func {
	/// Function signature
	signature: Cow<'static, str>,

	/// Description
	desc: Cow<'static, str>,

	/// Start position
	start_pos: Pos,

	/// End position
	end_pos: Pos,
}

impl Func {
	/// All currently known functions
	pub const FUNCTIONS: &'static [Self] = &[
		Self {
			signature: Cow::Borrowed("void InitHeap(int* addr, unsigned int size)"),
			desc:      Cow::Borrowed("Calls A(0x39)"),
			start_pos: Pos(0x8006a734),
			end_pos:   Pos(0x8006a744),
		},
		Self {
			signature: Cow::Borrowed("void start(void)"),
			desc:      Cow::Borrowed(indoc! {"
				Executable start.
				Zeroes out 0x80077a08..0x801ddf38.
				Initializes the stack, frame and global pointer.
				Calls InitHeap(0x8007f988, ???).
				Calls func_1025(0x8007f98c).
				Calls func_1026(string_0, string_0).
			"}),
			start_pos: Pos(0x80056270),
			end_pos:   Pos(0x80056328),
		},
		Self {
			signature: Cow::Borrowed("void func_1025(int*)"),
			desc:      Cow::Borrowed(indoc! {"
				At the end, calls func_446 indefinitely.
			"}),
			start_pos: Pos(0x80013e4c),
			end_pos:   Pos(0x80013f00),
		},
		Self {
			signature: Cow::Borrowed("int func_446(int)"),
			desc:      Cow::Borrowed(indoc! {"
				
			"}),
			start_pos: Pos(0x80069124),
			end_pos:   Pos(0x80069150),
		},
	];
}
