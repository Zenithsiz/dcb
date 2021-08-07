//! Instruction
//!
//! Instructions of the msd format are variable length, they are word-aligned.
//!
//! The first word of the instruction is the mnemonic, with the words following being data.

use std::assert_matches::assert_matches;

use byteorder::{ByteOrder, LittleEndian};

use crate::ComboBox;


/// Command
#[derive(PartialEq, Clone, Debug)]
pub enum Command<'a> {
	/// Display buffer
	DisplayBuffer,

	/// Wait for input
	WaitInput,

	/// Clear screen
	ClearScreen,

	/// Finish combo box
	FinishComboBox,

	DisplayBattleCafe,
	DisplayPlayerRoom,
	DisplayCardList,
	DisplayChoosePartner,
	DisplayBattleArena,
	DisplayKeyboard,
	DisplayEditPartner,
	DisplayTextBox,

	/// Set value
	SetValue {
		var:    u16,
		op:     u32,
		value1: u32,
	},

	/// Reset
	Reset,

	/// Test
	Test {
		var:    u16,
		value1: u32,
		value2: u32,
	},

	/// Jump
	Jump {
		var:  u16,
		addr: u32,
	},

	/// Unknown 0a
	Unknown0a {
		value: u16,
	},

	/// Open combo box
	OpenComboBox {
		combo_box: ComboBox,
	},

	/// Add combo box option
	AddComboBoxOption {
		value: u16,
	},

	/// Display scene
	DisplayScene {
		value0: u8,
		value1: u16,
	},

	/// Set buffer
	SetBuffer {
		kind:  u8,
		bytes: &'a [u8],
	},

	/// Set brightness
	SetBrightness {
		kind:       u8,
		place:      u16,
		brightness: u16,
		value:      u16,
	},
}

impl<'a> Command<'a> {
	/// Parses a command
	pub fn parse(slice: &'a [u8]) -> Option<Self> {
		let command = match *slice.get(..0x4)? {
			[0x0a, 0x0, 0x01, 0x0] => Self::FinishComboBox,
			[0x0a, 0x0, 0x02, 0x0] => Self::DisplayBattleCafe,
			[0x0a, 0x0, 0x04, 0x0] => Self::DisplayBuffer,
			[0x0a, 0x0, 0x05, 0x0] => Self::WaitInput,
			[0x0a, 0x0, 0x06, 0x0] => Self::ClearScreen,
			[0x0a, 0x0, 0x07, 0x0] => Self::DisplayPlayerRoom,
			[0x0a, 0x0, 0x09, 0x0] => Self::DisplayCardList,
			[0x0a, 0x0, 0x0a, 0x0] => Self::DisplayChoosePartner,
			[0x0a, 0x0, 0x0c, 0x0] => Self::DisplayBattleArena,
			[0x0a, 0x0, 0x0f, 0x0] => Self::DisplayKeyboard,
			[0x0a, 0x0, 0x11, 0x0] => Self::DisplayEditPartner,
			[0x0a, 0x0, 0x16, 0x0] => Self::DisplayTextBox,
			[0x0a, 0x0, value0, value1] => Self::Unknown0a {
				value: LittleEndian::read_u16(&[value0, value1]),
			},

			// Reset
			// Maybe var = `0x0` is the program counter?
			// Played around with this idea, but couldn't jump anywhere other than 0 even with dividing positions by 4
			[0x07, 0x0, 0x0, 0x0] if slice.get(0x4..0xc)? == [0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0] => Self::Reset,

			// Set variable
			[0x07, 0x0, var0, var1] => {
				let var = LittleEndian::read_u16(&[var0, var1]);
				let op = LittleEndian::read_u32(slice.get(0x4..0x8)?);
				let value1 = LittleEndian::read_u32(slice.get(0x8..0xc)?);

				// 0 => Set
				// 1 => Add
				// 6 => ???

				assert_matches!(op, 0 | 1 | 6, "Unknown set_value operation");

				Self::SetValue { var, op, value1 }
			},

			// Test
			[0x09, 0x0, var0, var1] => {
				let var = LittleEndian::read_u16(&[var0, var1]);
				let value1 = LittleEndian::read_u32(slice.get(0x4..0x8)?);
				let value2 = LittleEndian::read_u32(slice.get(0x8..0xc)?);

				assert_matches!(value1, 3 | 5, "Unknown test value1");

				Self::Test { var, value1, value2 }
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

				Self::AddComboBoxOption { value }
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
					kind,
					bytes: &bytes[..(len - 1)],
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
	pub fn size(&self) -> usize {
		match self {
			Command::DisplayBuffer => 4,
			Command::WaitInput => 4,
			Command::ClearScreen => 4,
			Command::FinishComboBox => 4,
			Command::DisplayBattleCafe => 4,
			Command::DisplayPlayerRoom => 4,
			Command::DisplayCardList => 4,
			Command::DisplayChoosePartner => 4,
			Command::DisplayBattleArena => 4,
			Command::DisplayKeyboard => 4,
			Command::DisplayEditPartner => 4,
			Command::DisplayTextBox => 4,
			Command::SetValue { .. } => 0xc,
			Command::Reset => 0xc,
			Command::Test { .. } => 0xc,
			Command::Jump { .. } => 8,
			Command::Unknown0a { .. } => 4,
			Command::OpenComboBox { .. } => 8,
			Command::AddComboBoxOption { .. } => 8,
			Command::DisplayScene { .. } => 8,
			Command::SetBuffer { bytes, .. } => {
				let len = bytes.len() + 2;
				4 + len + (4 - len % 4)
			},
			Command::SetBrightness { .. } => 16,
		}
	}
}
