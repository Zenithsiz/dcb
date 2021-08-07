//! Instruction
//!
//! Instructions of the msd format are variable length, they are word-aligned.
//!
//! The first word of the instruction is the mnemonic, with the words following being data.

// Modules
mod error;

// Exports
pub use error::EncodeError;

// Imports
use crate::{ComboBox, Screen};
use byteorder::{ByteOrder, LittleEndian};
use std::{assert_matches::assert_matches, io};
use zutil::TryIntoAs;

/// Instruction
// TODO: Merge common instructions
#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum Inst<'a> {
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

	/// Opens a screen
	OpenScreen(Screen),

	/// Sets the background to the battle arena
	// TODO: Check what texture it uses, looks all messed up most of the times.
	SetBgBattleArena,

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
		value0: u16,

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
		buffer: u16,

		/// The bytes to set
		bytes: &'a [u8],
	},

	/// Sets the brightness of `place` to `brightness`.
	// TODO: Figure out the rest
	SetBrightness {
		/// Unknown
		kind: u16,

		/// Place
		place: u16,

		/// Brightness
		brightness: u16,

		/// Unknown
		value: u16,
	},
}

impl<'a> Inst<'a> {
	/// Decodes an instruction
	#[must_use]
	#[allow(clippy::too_many_lines)] // TODO: Simplify
	pub fn decode(slice: &'a [u8]) -> Option<Self> {
		let inst = match LittleEndian::read_u16(slice.get(..0x2)?) {
			// Jump
			0x05 => {
				let var = LittleEndian::read_u16(slice.get(0x2..0x4)?);
				let addr = LittleEndian::read_u32(slice.get(0x4..0x8)?);

				Self::Jump { var, addr }
			},

			// Change variable
			0x07 => {
				let var = LittleEndian::read_u16(slice.get(0x2..0x4)?);
				let op = LittleEndian::read_u32(slice.get(0x4..0x8)?);
				let value = LittleEndian::read_u32(slice.get(0x8..0xc)?);

				// 0 => Set, 1 => Add, 6 => ???

				assert_matches!(op, 0 | 1 | 6, "Unknown set_value operation");

				Self::ChangeVar { var, op, value }
			},

			// Set buffer
			0x08 => {
				let buffer = LittleEndian::read_u16(slice.get(0x2..0x4)?);
				let len = usize::from(LittleEndian::read_u16(slice.get(0x4..0x6)?));
				let bytes = slice.get(0x6..(0x6 + len))?;

				// If any bytes except the last are null or the last isn't null, return `None`.
				if bytes.iter().take(len.checked_sub(1)?).any(|&ch| ch == 0) {
					return None;
				}
				if *bytes.get(len.checked_sub(1)?)? != 0 {
					return None;
				}

				Self::SetBuffer {
					buffer,
					bytes: &bytes[..(len - 1)],
				}
			},

			// Test
			0x09 => {
				let var = LittleEndian::read_u16(slice.get(0x2..0x4)?);
				let op = LittleEndian::read_u32(slice.get(0x4..0x8)?);
				let value = LittleEndian::read_u32(slice.get(0x8..0xc)?);

				assert_matches!(op, 3 | 5, "Unknown test operation");

				Self::Test { var, op, value }
			},

			// Misc.
			0x0a => match LittleEndian::read_u16(slice.get(0x2..0x4)?) {
				0x01 => Self::ComboBoxAwait,
				0x02 => Self::SetBgBattleCafe,
				0x04 => Self::DisplayTextBuffer,
				0x05 => Self::WaitInput,
				0x06 => Self::EmptyTextBox,
				0x07 => Self::OpenScreen(Screen::PlayerRoom),
				0x09 => Self::OpenScreen(Screen::CardList),
				0x0a => Self::OpenScreen(Screen::ChoosePartner),
				0x0c => Self::SetBgBattleArena,
				0x0f => Self::OpenScreen(Screen::Keyboard),
				0x11 => Self::OpenScreen(Screen::EditPartner),
				0x16 => Self::DisplayCenterTextBox,
				value => Self::Unknown0a { value },
			},

			// ???
			0x0b => match LittleEndian::read_u16(slice.get(0x2..0x4)?) {
				// Open combo box
				0x0 => {
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

				// Add combo box button
				0x1 => {
					if slice.get(0x4..0x6)? != [0x0, 0x0] {
						return None;
					}
					let value = LittleEndian::read_u16(slice.get(0x6..0x8)?);

					Self::AddComboBoxButton { value }
				},

				// Display scene?
				value0 => {
					if slice.get(0x4..0x6)? != [0x0, 0x0] {
						return None;
					}
					let value1 = LittleEndian::read_u16(slice.get(0x6..0x8)?);

					Self::DisplayScene { value0, value1 }
				},
			},

			// Set brightness
			0x0d => {
				let kind = LittleEndian::read_u16(slice.get(0x2..0x4)?);
				let place = LittleEndian::read_u16(slice.get(0x6..0x8)?);
				let brightness = LittleEndian::read_u16(slice.get(0xa..0xc)?);
				let value = LittleEndian::read_u16(slice.get(0xe..0x10)?);

				// If any of the padding is non-zero, return
				if slice.get(0x4..0x6)? != [0x0, 0x0] ||
					slice.get(0x8..0xa)? != [0x0, 0x0] ||
					slice.get(0xc..0xe)? != [0x0, 0x0]
				{
					return None;
				}

				Self::SetBrightness {
					kind,
					place,
					brightness,
					value,
				}
			},
			_ => return None,
		};

		Some(inst)
	}

	/// Encodes this instruction
	// TODO: Improve
	pub fn encode<W: io::Write>(&self, f: &mut W) -> Result<(), EncodeError> {
		match self {
			Inst::DisplayTextBuffer => f.write_all(&[0xa, 0x0, 0x4, 0x0])?,
			Inst::WaitInput => f.write_all(&[0xa, 0x0, 0x5, 0x0])?,
			Inst::EmptyTextBox => f.write_all(&[0xa, 0x0, 0x6, 0x0])?,
			Inst::SetBgBattleCafe => f.write_all(&[0xa, 0x0, 0x2, 0x0])?,
			Inst::OpenScreen(screen) => match screen {
				Screen::PlayerRoom => f.write_all(&[0xa, 0x0, 0x7, 0x0])?,
				Screen::CardList => f.write_all(&[0xa, 0x0, 0x9, 0x0])?,
				Screen::ChoosePartner => f.write_all(&[0xa, 0x0, 0xa, 0x0])?,
				Screen::EditPartner => f.write_all(&[0xa, 0x0, 0x11, 0x0])?,
				Screen::Keyboard => f.write_all(&[0xa, 0x0, 0xf, 0x0])?,
			},
			Inst::SetBgBattleArena => f.write_all(&[0xa, 0x0, 0xc, 0x0])?,
			Inst::DisplayCenterTextBox => f.write_all(&[0xa, 0x0, 0x16, 0x0])?,
			Inst::ChangeVar { var, op, value } => {
				f.write_all(&[0x7, 0x0])?;
				f.write_all(&var.to_le_bytes())?;
				f.write_all(&op.to_le_bytes())?;
				f.write_all(&value.to_le_bytes())?;
			},
			Inst::Test { var, op, value } => {
				f.write_all(&[0x9, 0x0])?;
				f.write_all(&var.to_le_bytes())?;
				f.write_all(&op.to_le_bytes())?;
				f.write_all(&value.to_le_bytes())?;
			},
			Inst::Jump { var, addr } => {
				f.write_all(&[0x5, 0x0])?;
				f.write_all(&var.to_le_bytes())?;
				f.write_all(&addr.to_le_bytes())?;
			},
			Inst::Unknown0a { value } => {
				f.write_all(&[0xa, 0x0])?;
				f.write_all(&value.to_le_bytes())?;
			},
			Inst::OpenComboBox { combo_box } => {
				f.write_all(&[0xb, 0x0, 0x0, 0x0])?;
				f.write_all(&[0x0, 0x0])?;
				f.write_all(
					&match combo_box {
						ComboBox::Small => 0x61u16,
						ComboBox::Large => 0x78,
					}
					.to_le_bytes(),
				)?;
			},
			Inst::AddComboBoxButton { value } => {
				f.write_all(&[0xb, 0x0, 0x1, 0x0])?;
				f.write_all(&[0x0, 0x0])?;
				f.write_all(&value.to_le_bytes())?;
			},
			Inst::ComboBoxAwait => f.write_all(&[0xa, 0x0, 0x1, 0x0])?,
			Inst::DisplayScene { value0, value1 } => {
				f.write_all(&[0xb, 0x0])?;
				f.write_all(&value0.to_be_bytes())?;
				f.write_all(&[0x0, 0x0])?;
				f.write_all(&value1.to_le_bytes())?;
			},
			Inst::SetBuffer { buffer, bytes } => {
				f.write_all(&[0x8, 0x0])?;
				f.write_all(&buffer.to_le_bytes())?;

				let len = bytes.len().try_into_as::<u16>().map_err(EncodeError::LenToU16)?;
				f.write_all(&len.to_le_bytes())?;
				f.write_all(bytes)?;

				let nulls_len = 4 - (bytes.len() + 2) % 4;
				f.write_all(&[0; 4][..nulls_len])?;
			},
			Inst::SetBrightness {
				kind,
				place,
				brightness,
				value,
			} => {
				f.write_all(&[0xd, 0x0])?;
				f.write_all(&kind.to_be_bytes())?;
				f.write_all(&[0x0, 0x0])?;
				f.write_all(&place.to_le_bytes())?;
				f.write_all(&[0x0, 0x0])?;
				f.write_all(&brightness.to_le_bytes())?;
				f.write_all(&[0x0, 0x0])?;
				f.write_all(&value.to_le_bytes())?;
			},
		}

		Ok(())
	}

	/// Returns this instruction's size
	#[must_use]
	pub const fn size(&self) -> usize {
		// TODO: Combine them
		#[allow(clippy::match_same_arms)] // We want to explicitly not combine them for now
		match self {
			Inst::DisplayTextBuffer => 4,
			Inst::WaitInput => 4,
			Inst::EmptyTextBox => 4,
			Inst::ComboBoxAwait => 4,
			Inst::SetBgBattleCafe => 4,
			Inst::OpenScreen(_) => 4,
			Inst::SetBgBattleArena => 4,
			Inst::DisplayCenterTextBox => 4,
			Inst::ChangeVar { .. } => 0xc,
			Inst::Test { .. } => 0xc,
			Inst::Jump { .. } => 8,
			Inst::Unknown0a { .. } => 4,
			Inst::OpenComboBox { .. } => 8,
			Inst::AddComboBoxButton { .. } => 8,
			Inst::DisplayScene { .. } => 8,
			Inst::SetBuffer { bytes, .. } => {
				let len = bytes.len() + 2;
				4 + len + (4 - len % 4)
			},
			Inst::SetBrightness { .. } => 16,
		}
	}
}
