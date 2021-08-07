//! Instruction
//!
//! Instructions of the msd format are variable length, they are word-aligned.
//!
//! The first word of the instruction is the mnemonic, with the words following being data.

// Imports
use crate::ComboBox;
use byteorder::{ByteOrder, LittleEndian};
use std::assert_matches::assert_matches;

/// Command
// TODO: Merge common commands
#[derive(PartialEq, Clone, Debug)]
pub enum Command<'a> {
	/// Displays the text buffer in the text box.
	///
	/// Displays the text in the text buffer and scrolls to the next line.
	/// If the text box is full, waits for input until displaying the next line.
	DisplayTextBuffer,

	/// Wait for input from the user.
	///
	/// Pauses execution until the users sends input.
	WaitInput,

	/// Empty text box
	///
	/// Empties the text box, removing all characters and setting
	/// the cursor to the first line.
	EmptyTextBox,

	/// Sets the background to the battle cafe
	SetBgBattleCafe,

	/// Sets the background to the battle arena
	// TODO: Check what texture it uses, looks all messed up most of the times.
	SetBgBattleArena,

	/// Opens the player room
	OpenPlayerRoom,

	/// Opens the card list
	OpenCardList,

	/// Opens the partner chooser screen
	// TODO: Figure out parameters for this, they're all veemon
	OpenChoosePartner,

	/// Opens the keyboard
	// TODO: Check where the text goes
	OpenKeyboard,

	/// Opens the partner edit screen
	OpenEditPartner,

	/// Opens the center text box
	// TODO: Rename, somewhat confusing
	DisplayCenterTextBox,

	/// Changes a variable value
	///
	/// Depending on `op`, either sets or adds `value` to the `var` variable
	// TODO: Make `op` an enum
	ChangeVar {
		/// Variable
		var: u16,

		/// Operation
		op: u32,

		/// Value
		value: u32,
	},

	/// Tests if a variable has a value
	///
	/// Depending on `op`, either checks if `var` is equal to, or less than `value`
	// TODO: Confirm less than
	// TODO: Explain that it skips the next instruction if false, maybe rename to `exec_if` or something
	Test {
		/// Variable
		var: u16,

		/// Operation
		op: u32,

		/// Value
		value: u32,
	},

	/// Jumps to `addr`
	// TODO: Figure out `var`, seems to somewhat coincide with the label number, but that would be weird
	Jump {
		/// Unknown
		var: u16,

		/// Address to jump to
		addr: u32,
	},

	/// Unknown `0a`.
	// TODO:
	Unknown0a {
		/// Unknown
		value: u16,
	},

	/// Opens a combo box
	OpenComboBox {
		/// The combo box being opened
		combo_box: ComboBox,
	},

	/// Adds a combo box button
	AddComboBoxButton {
		/// The button value
		value: u16,
	},

	/// Awaits the user's selection on the combo box
	ComboBoxAwait,

	/// Display scene
	// TODO: Figure out
	DisplayScene {
		/// Unknown
		value0: u8,

		/// Unknown
		value1: u16,
	},

	/// Sets buffer `buffer` to `bytes`.
	///
	/// The following are the known buffers:
	/// - 0x4: Text buffer
	// TODO: Have `buffer` be an enum of the buffers and move the explanation there
	SetBuffer {
		/// The buffer to set
		buffer: u8,

		/// The bytes to set
		bytes: &'a [u8],
	},

	/// Sets the brightness of `place` to `brightness`.
	// TODO: Figure out the rest
	SetBrightness {
		/// Unknown
		kind: u8,

		/// Place
		place: u16,

		/// Brightness
		brightness: u16,

		/// Unknown
		value: u16,
	},
}

impl<'a> Command<'a> {
	/// Parses a command from a slice of bytes.
	///
	/// Ignores everything after the command.
	#[must_use]
	#[allow(clippy::too_many_lines)] // TODO: Simplify
	pub fn parse(slice: &'a [u8]) -> Option<Self> {
		let command = match *slice.get(..0x4)? {
			[0x0a, 0x0, 0x01, 0x0] => Self::ComboBoxAwait,
			[0x0a, 0x0, 0x02, 0x0] => Self::SetBgBattleCafe,
			[0x0a, 0x0, 0x04, 0x0] => Self::DisplayTextBuffer,
			[0x0a, 0x0, 0x05, 0x0] => Self::WaitInput,
			[0x0a, 0x0, 0x06, 0x0] => Self::EmptyTextBox,
			[0x0a, 0x0, 0x07, 0x0] => Self::OpenPlayerRoom,
			[0x0a, 0x0, 0x09, 0x0] => Self::OpenCardList,
			[0x0a, 0x0, 0x0a, 0x0] => Self::OpenChoosePartner,
			[0x0a, 0x0, 0x0c, 0x0] => Self::SetBgBattleArena,
			[0x0a, 0x0, 0x0f, 0x0] => Self::OpenKeyboard,
			[0x0a, 0x0, 0x11, 0x0] => Self::OpenEditPartner,
			[0x0a, 0x0, 0x16, 0x0] => Self::DisplayCenterTextBox,
			[0x0a, 0x0, value0, value1] => Self::Unknown0a {
				value: LittleEndian::read_u16(&[value0, value1]),
			},

			// Set variable
			[0x07, 0x0, var0, var1] => {
				let var = LittleEndian::read_u16(&[var0, var1]);
				let op = LittleEndian::read_u32(slice.get(0x4..0x8)?);
				let value1 = LittleEndian::read_u32(slice.get(0x8..0xc)?);

				// 0 => Set
				// 1 => Add
				// 6 => ???

				assert_matches!(op, 0 | 1 | 6, "Unknown set_value operation");

				Self::ChangeVar { var, op, value: value1 }
			},

			// Test
			[0x09, 0x0, var0, var1] => {
				let var = LittleEndian::read_u16(&[var0, var1]);
				let value1 = LittleEndian::read_u32(slice.get(0x4..0x8)?);
				let value2 = LittleEndian::read_u32(slice.get(0x8..0xc)?);

				assert_matches!(value1, 3 | 5, "Unknown test value1");

				Self::Test {
					var,
					op: value1,
					value: value2,
				}
			},

			// Jump?
			[0x05, 0x0, var0, var1] => {
				let var = LittleEndian::read_u16(&[var0, var1]);
				let addr = LittleEndian::read_u32(slice.get(0x4..0x8)?);

				Self::Jump { var, addr }
			},

			// Open combo box
			[0x0b, 0x0, 0x0, 0x0] => {
				if slice.get(0x4..0x6)? != [0x0, 0x0] {
					return None;
				}
				let value = LittleEndian::read_u16(slice.get(0x6..0x8)?);

				// value: 0x61 0x78
				let combo_box = match value {
					0x61 => ComboBox::Small,
					0x78 => ComboBox::Large,
					_ => return None,
				};

				Self::OpenComboBox { combo_box }
			},
			[0x0b, 0x0, 0x1, 0x0] => {
				if slice.get(0x4..0x6)? != [0x0, 0x0] {
					return None;
				}
				let value = LittleEndian::read_u16(slice.get(0x6..0x8)?);

				Self::AddComboBoxButton { value }
			},
			// Display scene?
			[0x0b, 0x0, value0, 0x0] => {
				if slice.get(0x4..0x6)? != [0x0, 0x0] {
					return None;
				}
				let value1 = LittleEndian::read_u16(slice.get(0x6..0x8)?);

				// If 0x2 is skipped, battle doesn't happen

				// value0: 0x2 0x3 0x4 0x6 0x7 0x8 0x9 0xa 0xc 0xd 0xe 0xf 0x10 0x11 0x12 0x13 0x14 0x15

				Self::DisplayScene { value0, value1 }
			},

			// Set buffer
			[0x08, 0x0, kind, 0x0] => {
				let len = usize::from(LittleEndian::read_u16(slice.get(0x4..0x6)?));
				if len == 0 {
					return None;
				}

				let bytes = slice.get(0x6..(0x6 + len))?;


				if bytes[0..(len - 1)].iter().any(|&ch| ch == 0) {
					return None;
				}
				if bytes[len - 1] != 0 {
					return None;
				}

				Self::SetBuffer {
					buffer: kind,
					bytes:  &bytes[..(len - 1)],
				}
			},

			// Set brightness
			[0x0d, 0x0, kind, 0x0] => {
				if slice.get(0x4..0x6)? != [0x0, 0x0] {
					return None;
				}
				let place = LittleEndian::read_u16(slice.get(0x6..0x8)?);
				if slice.get(0x8..0xa)? != [0x0, 0x0] {
					return None;
				}
				let brightness = LittleEndian::read_u16(slice.get(0xa..0xc)?);
				if slice.get(0xc..0xe)? != [0x0, 0x0] {
					return None;
				}
				let value = LittleEndian::read_u16(slice.get(0xe..0x10)?);

				Self::SetBrightness {
					kind,
					place,
					brightness,
					value,
				}
			},

			_ => return None,
		};

		Some(command)
	}

	/// Returns this command's size
	#[must_use]
	pub const fn size(&self) -> usize {
		// TODO: Combine them
		#[allow(clippy::match_same_arms)] // We want to explicitly not combine them for now
		match self {
			Command::DisplayTextBuffer => 4,
			Command::WaitInput => 4,
			Command::EmptyTextBox => 4,
			Command::ComboBoxAwait => 4,
			Command::SetBgBattleCafe => 4,
			Command::OpenPlayerRoom => 4,
			Command::OpenCardList => 4,
			Command::OpenChoosePartner => 4,
			Command::SetBgBattleArena => 4,
			Command::OpenKeyboard => 4,
			Command::OpenEditPartner => 4,
			Command::DisplayCenterTextBox => 4,
			Command::ChangeVar { .. } => 0xc,
			Command::Test { .. } => 0xc,
			Command::Jump { .. } => 8,
			Command::Unknown0a { .. } => 4,
			Command::OpenComboBox { .. } => 8,
			Command::AddComboBoxButton { .. } => 8,
			Command::DisplayScene { .. } => 8,
			Command::SetBuffer { bytes, .. } => {
				let len = bytes.len() + 2;
				4 + len + (4 - len % 4)
			},
			Command::SetBrightness { .. } => 16,
		}
	}
}
