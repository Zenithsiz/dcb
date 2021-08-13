//! Instruction
//!
//! Instructions of the msd format are variable length, they are word-aligned.
//!
//! The first word of the instruction is the mnemonic, with the words following being data.

// Modules
mod display;
mod error;

// Exports
pub use display::DisplayCtx;
pub use error::{DisplayError, EncodeError};

// Imports
use crate::{ComboBox, Screen};
use byteorder::{ByteOrder, LittleEndian};
use encoding_rs::SHIFT_JIS;
use itertools::Itertools;
use std::{assert_matches::assert_matches, io};
use zutil::TryIntoAs;

/// Instruction
// TODO: Merge common instructions
#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
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

	/// Resets the game completion to 0 points.
	ResetGameCompletion,

	/// Transitions between the single red flashing light at the beginning
	/// of each city to the proper menu interface
	StartTransition,

	/// Resets all of the partners in the choose partner menu.
	///
	/// Partners may then be added with `DisplayScene 0xa`
	ResetChoosePartner,

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
		// TODO: Just have the value here for now
		combo_box: ComboBox,
	},

	/// Adds a combo box button
	AddComboBoxButton {
		/// The button value
		value: u16,
	},

	/// Awaits the user's selection on the combo box
	ComboBoxAwait,

	/// Awaits the user's selection on the battle cafe
	BattleCafeAwait,

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
		#[serde(with = "serde_shift_jis_str")]
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
				0x00 => Self::StartTransition,
				0x01 => Self::ComboBoxAwait,
				0x02 => Self::SetBgBattleCafe,
				0x03 => Self::BattleCafeAwait,
				0x04 => Self::DisplayTextBuffer,
				0x05 => Self::WaitInput,
				0x06 => Self::EmptyTextBox,
				0x07 => Self::OpenScreen(Screen::PlayerRoom),
				0x09 => Self::OpenScreen(Screen::CardList),
				0x0a => Self::OpenScreen(Screen::ChoosePartner),
				0x0b => Self::ResetChoosePartner,
				0x0c => Self::SetBgBattleArena,
				0x0d => Self::OpenScreen(Screen::BattleArenaIntro),
				0x0f => Self::OpenScreen(Screen::Keyboard),
				0x11 => Self::OpenScreen(Screen::EditPartner),
				0x13 => Self::ResetGameCompletion,
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
			Self::DisplayTextBuffer => f.write_all(&[0xa, 0x0, 0x4, 0x0])?,
			Self::WaitInput => f.write_all(&[0xa, 0x0, 0x5, 0x0])?,
			Self::EmptyTextBox => f.write_all(&[0xa, 0x0, 0x6, 0x0])?,
			Self::SetBgBattleCafe => f.write_all(&[0xa, 0x0, 0x2, 0x0])?,
			Self::OpenScreen(screen) => match screen {
				Screen::PlayerRoom => f.write_all(&[0xa, 0x0, 0x7, 0x0])?,
				Screen::CardList => f.write_all(&[0xa, 0x0, 0x9, 0x0])?,
				Screen::ChoosePartner => f.write_all(&[0xa, 0x0, 0xa, 0x0])?,
				Screen::EditPartner => f.write_all(&[0xa, 0x0, 0x11, 0x0])?,
				Screen::Keyboard => f.write_all(&[0xa, 0x0, 0xf, 0x0])?,
				Screen::BattleArenaIntro => f.write_all(&[0xa, 0x0, 0xd, 0x0])?,
			},
			Self::SetBgBattleArena => f.write_all(&[0xa, 0x0, 0xc, 0x0])?,
			Self::DisplayCenterTextBox => f.write_all(&[0xa, 0x0, 0x16, 0x0])?,
			Self::ResetGameCompletion => f.write_all(&[0xa, 0x0, 0x13, 0x0])?,
			Self::StartTransition => f.write_all(&[0xa, 0x0, 0x0, 0x0])?,
			Self::ResetChoosePartner => f.write_all(&[0xa, 0x0, 0xb, 0x0])?,
			Self::ChangeVar { var, op, value } => {
				f.write_all(&[0x7, 0x0])?;
				f.write_all(&var.to_le_bytes())?;
				f.write_all(&op.to_le_bytes())?;
				f.write_all(&value.to_le_bytes())?;
			},
			Self::Test { var, op, value } => {
				f.write_all(&[0x9, 0x0])?;
				f.write_all(&var.to_le_bytes())?;
				f.write_all(&op.to_le_bytes())?;
				f.write_all(&value.to_le_bytes())?;
			},
			Self::Jump { var, addr } => {
				f.write_all(&[0x5, 0x0])?;
				f.write_all(&var.to_le_bytes())?;
				f.write_all(&addr.to_le_bytes())?;
			},
			Self::Unknown0a { value } => {
				f.write_all(&[0xa, 0x0])?;
				f.write_all(&value.to_le_bytes())?;
			},
			Self::OpenComboBox { combo_box } => {
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
			Self::AddComboBoxButton { value } => {
				f.write_all(&[0xb, 0x0, 0x1, 0x0])?;
				f.write_all(&[0x0, 0x0])?;
				f.write_all(&value.to_le_bytes())?;
			},
			Self::ComboBoxAwait => f.write_all(&[0xa, 0x0, 0x1, 0x0])?,
			Self::BattleCafeAwait => f.write_all(&[0xa, 0x0, 0x3, 0x0])?,
			Self::DisplayScene { value0, value1 } => {
				f.write_all(&[0xb, 0x0])?;
				f.write_all(&value0.to_be_bytes())?;
				f.write_all(&[0x0, 0x0])?;
				f.write_all(&value1.to_le_bytes())?;
			},
			Self::SetBuffer { buffer, bytes } => {
				f.write_all(&[0x8, 0x0])?;
				f.write_all(&buffer.to_le_bytes())?;

				let len = bytes.len().try_into_as::<u16>().map_err(EncodeError::LenToU16)?;
				f.write_all(&len.to_le_bytes())?;
				f.write_all(bytes)?;

				let nulls_len = 4 - (bytes.len() + 2) % 4;
				f.write_all(&[0; 4][..nulls_len])?;
			},
			Self::SetBrightness {
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

	/// Displays an instruction
	#[allow(clippy::too_many_lines)] // TODO: Refactor
	pub fn display<W: io::Write, Ctx: DisplayCtx>(&self, f: &mut W, ctx: &Ctx) -> Result<(), DisplayError> {
		match self {
			Self::DisplayTextBuffer => write!(f, "display_text_buffer")?,
			Self::WaitInput => write!(f, "wait_input")?,
			Self::EmptyTextBox => write!(f, "empty_text_box")?,
			Self::SetBgBattleCafe => write!(f, "set_bg \"Battle Cafe\"")?,
			Self::OpenScreen(screen) => write!(f, "open_screen \"{}\"", screen.as_str().escape_debug())?,
			Self::SetBgBattleArena => write!(f, "set_bg \"Battle Arena\"")?,
			Self::DisplayCenterTextBox => write!(f, "display_center_text_box")?,
			Self::ResetGameCompletion => write!(f, "reset_game_completion")?,
			Self::StartTransition => write!(f, "start_transition")?,
			Self::ResetChoosePartner => write!(f, "reset_choose_partner")?,
			Self::ChangeVar { var, op, value } => {
				let var_label = zutil::DisplayWrapper::new(|f| match ctx.var_label(*var) {
					Some(label) => write!(f, "{label}"),
					None => write!(f, "{var:#x}"),
				});

				match (var, op) {
					// Arena intro colors
					(0x5, 0) => {
						#[derive(derive_more::Display)]
						#[allow(clippy::missing_docs_in_private_items)] // Just for formatting
						enum Color {
							Yellow,
							Black,
							Green,
							Blue,
							Red,
						}

						let colors = self::bits_iterator(*value)
							.take(5)
							.zip([Color::Yellow, Color::Black, Color::Green, Color::Blue, Color::Red])
							.filter_map(|(set, color)| set.then_some(color));

						write!(f, "set_arena_match_intro_colors {}", colors.format("+"))?;
					},

					// Arena match intro index
					(0xb, 0) => {
						let idx = zutil::DisplayWrapper::new(|f| match value {
							0 => write!(f, "1st"),
							1 => write!(f, "2nd"),
							2 => write!(f, "3rd"),
							3 => write!(f, "4th"),
							4 => write!(f, "5th"),
							5 => write!(f, "Last"),
							_ => write!(f, "{value:#x}"),
						});

						write!(f, "set_arena_match_intro_idx \"{idx}\"")?;
					},

					(_, 0) => write!(f, "set_var {var_label}, {value:#x}")?,
					(_, 1) => write!(f, "add_var {var_label}, {value:#x}")?,
					(_, 6) => write!(f, "???_var {var_label}, {value:#x}")?,
					_ => unreachable!(),
				}
			},
			Self::Test { var, op, value } => {
				let var = zutil::DisplayWrapper::new(|f| match ctx.var_label(*var) {
					Some(label) => write!(f, "{label}"),
					None => write!(f, "{var:#x}"),
				});

				match op {
					3 => write!(f, "test_eq {var}, {value:#x}")?,
					5 => write!(f, "test_lt {var}, {value:#x}")?,
					_ => unreachable!(),
				}
			},

			Self::Jump { var, addr } => {
				let addr = zutil::DisplayWrapper::new(|f| match ctx.pos_label(*addr) {
					Some(label) => write!(f, "{label}"),
					None => write!(f, "{addr:#x}"),
				});

				write!(f, "jump {var:#x}, {addr}")?;
			},
			Self::Unknown0a { value } => write!(f, "unknown_0a {value:#x}")?,
			Self::OpenComboBox { combo_box: menu } => write!(f, "open_menu {}", menu.as_str())?,
			Self::DisplayScene { value0, value1 } => match (value0, value1) {
				(0x2, value) => write!(f, "battle {value:#x}")?,
				(0x8, location) => {
					let location = zutil::DisplayWrapper::new(|f| match location {
						0 => write!(f, "Player Room"),
						1 => write!(f, "Battle Cafe"),
						2 => write!(f, "Battle Arena"),
						3 => write!(f, "Extra Arena"),
						4 => write!(f, "Beet Arena"),
						5 => write!(f, "Haunted Arena"),
						_ => write!(f, "<Unknown arena {location:#x}>"),
					});

					write!(f, "display_location \"{location}\"")?;
				},
				(0xa, partner) => {
					let partner = zutil::DisplayWrapper::new(|f| match partner {
						0 => write!(f, "Veemon"),
						1 => write!(f, "Hawkmon"),
						2 => write!(f, "Armadillomon"),
						3 => write!(f, "Gatomon"),
						4 => write!(f, "Patamon"),
						5 => write!(f, "Wormmon"),
						_ => write!(f, "<Unknown partner {partner:#x}>"),
					});

					write!(f, "add_partner \"{partner}\"")?;
				},
				(0x12, value) => write!(f, "add_completion_points {value}")?,

				_ => write!(f, "display_scene {value0:#x}, {value1:#x}")?,
			},
			Self::SetBuffer { buffer, bytes } => {
				let bytes = zutil::DisplayWrapper::new(|f| {
					match SHIFT_JIS.decode_without_bom_handling_and_without_replacement(bytes) {
						Some(s) => {
							write!(f, "\"{}\"", s.escape_debug())
						},
						None => {
							let bytes = bytes.iter().format_with("", |byte, f| f(&format_args!("{byte:x}")));
							write!(f, "0x{bytes}",)
						},
					}
				});

				match buffer {
					0x4 => write!(f, "set_text_buffer {bytes}")?,
					_ => write!(f, "set_buffer {buffer:#x}, {bytes}")?,
				}
			},

			Self::SetBrightness {
				kind,
				place,
				brightness,
				value,
			} => match (kind, place, brightness, value) {
				(0x0, 0x0, _, 0xa) => write!(f, "set_light_left_char {brightness:#x}")?,
				(0x0, 0x1, _, 0xa) => write!(f, "set_light_right_char {brightness:#x}")?,
				(0x1, _, 0xffff, 0xffff) => write!(f, "set_light_unknown {place:#x}")?,
				_ => write!(f, "set_light {kind:#x}, {place:#x}, {brightness:#x}, {value:#x}")?,
			},
			Self::ComboBoxAwait => write!(f, "combo_box_await")?,
			Self::BattleCafeAwait => write!(f, "battle_cafe_await")?,
			// TODO: Not do it hackily like this
			Self::AddComboBoxButton { value } => match ComboBox::Small
				.parse_button(*value)
				.or_else(|| ComboBox::Large.parse_button(*value))
			{
				Some(button) => write!(f, "combo_box_add_button \"{}\"", button.as_str().escape_debug())?,
				None => write!(f, "combo_box_add_button {value:#x}")?,
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
			Self::DisplayTextBuffer => 4,
			Self::WaitInput => 4,
			Self::EmptyTextBox => 4,
			Self::ComboBoxAwait => 4,
			Self::BattleCafeAwait => 4,
			Self::SetBgBattleCafe => 4,
			Self::OpenScreen(_) => 4,
			Self::SetBgBattleArena => 4,
			Self::DisplayCenterTextBox => 4,
			Self::ResetGameCompletion => 4,
			Self::StartTransition => 4,
			Self::ResetChoosePartner => 4,
			Self::ChangeVar { .. } => 0xc,
			Self::Test { .. } => 0xc,
			Self::Jump { .. } => 8,
			Self::Unknown0a { .. } => 4,
			Self::OpenComboBox { .. } => 8,
			Self::AddComboBoxButton { .. } => 8,
			Self::DisplayScene { .. } => 8,
			Self::SetBuffer { bytes, .. } => {
				let len = bytes.len() + 2;
				4 + len + (4 - len % 4)
			},
			Self::SetBrightness { .. } => 16,
		}
	}
}


/// Helper module to serialize and deserialize bytes as `shift_jis`
mod serde_shift_jis_str {
	use std::borrow::Cow;

	// Imports
	use encoding_rs::SHIFT_JIS;
	use serde::{Deserialize, Deserializer, Serializer};

	/// Serialize
	pub fn serialize<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		// TODO: Not panic here, not sure how to get a `S::Error` though?
		let s = SHIFT_JIS
			.decode_without_bom_handling_and_without_replacement(bytes)
			.expect("Unable to decode as `SHIFT_JIS`");

		serializer.serialize_str(&*s)
	}

	/// Deserialize
	pub fn deserialize<'de, D>(deserializer: D) -> Result<&'de [u8], D::Error>
	where
		D: Deserializer<'de>,
	{
		// TODO: Not panic on bad encoding + non-borrowed encoding
		let s = <&str>::deserialize(deserializer)?;
		let (s, ..) = SHIFT_JIS.encode(s);

		match s {
			Cow::Borrowed(s) => Ok(s),
			Cow::Owned(_) => panic!("Unable to deserialize"),
		}
	}
}


/// Bits iterator from least to most significant
fn bits_iterator(mut value: u32) -> impl Iterator<Item = bool> {
	std::iter::from_fn(move || {
		let is_set = (value & 0x1) != 0;
		value >>= 1u32;
		Some(is_set)
	})
	.take(32)
}
