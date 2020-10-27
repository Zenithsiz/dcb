//! Executable functions

// Imports
use crate::game::exe::Pos;

/// A function within the executable
#[derive(PartialEq, Eq, Clone, Hash, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Func<S: AsRef<str>, C: AsRef<[(Pos, S)]>> {
	/// Function signature
	signature: S,

	/// Description
	desc: S,

	/// Comments
	comments: C,

	/// Start position
	start_pos: Pos,

	/// End position (non-inclusive)
	end_pos: Pos,
}

impl Func<&'static str, &'static [(Pos, &'static str)]> {
	/// List of all known functions
	pub const ALL: &'static [Self] = &[
		Self {
			signature: "void InitHeap(int* addr, unsigned int size)",
			desc:      "Calls A(0x39)",
			comments:  &[],
			start_pos: Pos(0x8006a734),
			end_pos:   Pos(0x8006a744),
		},
		Self {
			signature: "void start(void)",
			desc:      "Executable start",
			comments:  &[
				(Pos(0x80056280), "Zero out 0x80077a08 .. 0x801ddf38 word by word."),
				(Pos(0x80056284), "^"),
				(Pos(0x80056288), "^"),
				(Pos(0x8005628c), "^"),
				(Pos(0x800562f8), "InitHeap(0x8007f988, ???)"),
				(Pos(0x8005630c), "func_1025(0x8007f98c)"),
				(Pos(0x80056324), "func_1026(string_0, string_0)"),
			],
			start_pos: Pos(0x80056270),
			end_pos:   Pos(0x80056330),
		},
		Self {
			signature: "void func_1025(int*)",
			desc:      "",
			comments:  &[(Pos(0x80013ef4), "Called indefinitely?"), (Pos(0x80013efc), "^ Due to this loop")],
			start_pos: Pos(0x80013e4c),
			end_pos:   Pos(0x80013f04),
		},
		Self {
			signature: "int func_446(int)",
			desc:      "",
			comments:  &[],
			start_pos: Pos(0x80069124),
			end_pos:   Pos(0x80069150),
		},
	];
}
