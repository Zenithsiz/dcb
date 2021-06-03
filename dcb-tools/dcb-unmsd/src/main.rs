//! `.Msd` extractor

// Features
#![feature(
	array_chunks,
	format_args_capture,
	bool_to_option,
	assert_matches,
	exact_size_is_empty,
	iter_advance_by,
	try_blocks,
	cow_is_borrowed
)]

// Modules
mod cli;

// Imports
use anyhow::Context;
use byteorder::{ByteOrder, LittleEndian};
use cli::CliData;
use encoding_rs::SHIFT_JIS;
use itertools::Itertools;
use std::{collections::HashMap, convert::TryInto, fs};


fn main() -> Result<(), anyhow::Error> {
	// Initialize the logger
	simplelog::TermLogger::init(
		log::LevelFilter::Info,
		simplelog::Config::default(),
		simplelog::TerminalMode::Stderr,
	)
	.expect("Unable to initialize logger");

	// Get all data from cli
	let cli_data = CliData::new();

	// Read the file
	let mut contents = fs::read(&cli_data.input_file).context("Unable to read file")?;

	// Skip header
	contents.drain(..0x10);

	// Parse all commands
	let commands = contents
		.iter()
		.batching(|it| {
			let pos = it.as_slice().as_ptr() as usize - contents.as_slice().as_ptr() as usize;
			let pos = match pos.try_into() {
				Ok(pos) => pos,
				Err(_) => return Some(Err(anyhow::anyhow!("Position {:#x} didn't fit into a `u32`", pos))),
			};
			match Command::parse(it.as_slice()) {
				Some(command) => {
					it.advance_by(command.size())
						.expect("Iterator had less elements than size of command");
					Some(Ok((pos, command)))
				},
				None => match it.is_empty() {
					true => None,
					false => Some(Err(anyhow::anyhow!(
						"Unable to parse command at {:#010x}: {:?}",
						pos,
						&it.as_slice()[..4]
					))),
				},
			}
		})
		.collect::<Result<Vec<_>, anyhow::Error>>()
		.context("Unable to parse commands")?;

	log::info!("Found {} commands", commands.len());

	// Get all jumps
	let mut last_label_number = 0;
	let mut next_label = |name| {
		let num = last_label_number;
		last_label_number += 1;
		format!("{name}_{num}")
	};
	let labels: HashMap<u32, String> = commands
		.iter()
		.filter_map(|&(pos, ref command)| match *command {
			Command::Jump { addr, .. } => Some((addr, next_label("jump"))),
			Command::DisplayScene {
				value0: 0xf | 0xe,
				value1,
			} => Some((pos + 4 * u32::from(value1), next_label("scene"))),
			_ => None,
		})
		.collect();

	let mut state = State::Start;
	for (pos, command) in commands {
		if let Some(label) = labels.get(&pos) {
			println!("{label}:");
		};

		state
			.parse_next(&labels, command)
			.with_context(|| format!("Unable to parse command at {pos:#010x} in current context"))?;
	}

	state.finish().context("Unable to finish state")?;

	Ok(())
}

/// State
#[derive(PartialEq, Clone, Debug)]
pub enum State {
	/// Start
	Start,

	/// Menu
	Menu {
		/// Current menu
		menu: Menu,

		/// All buttons
		buttons: Vec<MenuButton>,
	},
}

impl State {
	/// Parses the next command
	pub fn parse_next(&mut self, labels: &HashMap<u32, String>, command: Command) -> Result<(), anyhow::Error> {
		match (&mut *self, command) {
			(State::Start, Command::DisplayBuffer) => println!("display_buffer"),
			(State::Start, Command::WaitInput) => println!("wait_input"),
			(State::Start, Command::NewScreen) => println!("new_screen"),
			(State::Start, Command::SetValue { var, value0, value1 }) => {
				println!("set_value {var:#x}, {value0:#x}, {value1:#x}")
			},
			(
				State::Start,
				Command::Unknown07 {
					value0,
					kind,
					value1,
					value2,
				},
			) => {
				println!("unknown_07 {value0:#x}, {kind:#x}, {value1:#x}, {value2:#x}")
			},
			(
				State::Start,
				Command::ChoiceJump {
					value0,
					kind,
					value1,
					value2,
				},
			) => {
				println!("menu_choice_offsets {value0:#x}, {kind:#x}, {value1:#x}, {value2:#x}")
			},
			(State::Start, Command::Jump { value, kind, addr }) => match labels.get(&addr) {
				Some(label) => println!("jump {value:#x}, {kind:#x}, {label}"),
				None => println!("jump {value:#x}, {kind:#x}, {addr:#010x}"),
			},
			(State::Start, Command::Unknown0a { value, kind }) => println!("unknown_0a {value:#x}, {kind:#x}"),
			(State::Start, Command::OpenMenu { menu }) => *self = State::Menu { menu, buttons: vec![] },
			(State::Start, Command::DisplayScene { value0, value1 }) => match (value0, value1) {
				(0x2, _) => println!("battle {value1:#x}"),

				(0xf, _) => println!("battle1 {value1:#x}"),
				(0xe, _) => println!("battle2 {value1:#x}"),
				_ => println!("display_scene {value0:#x}, {value1:#x}"),
			},
			(State::Start, Command::SetBuffer { kind, bytes }) => {
				let s = SHIFT_JIS
					.decode_without_bom_handling_and_without_replacement(bytes)
					.context("Unable to parse text buffer as utf-8")?;

				match kind {
					0x4 => println!("set_text_buffer \"{}\"", s.escape_debug()),
					_ => println!("set_buffer {kind:#x}, \"{}\"", s.escape_debug()),
				}
			},
			(
				State::Start,
				Command::SetBrightness {
					kind,
					place,
					brightness,
					value,
				},
			) => match (kind, place, brightness, value) {
				(0x0, 0x0, _, 0xa) => println!("set_light_left_char {brightness:#x}"),
				(0x0, 0x1, _, 0xa) => println!("set_light_right_char {brightness:#x}"),
				(0x1, _, 0xffff, 0xffff) => println!("set_light_unknown {place:#x}"),
				_ => println!("set_light {kind:#x}, {place:#x}, {brightness:#x}, {value:#x}"),
			},
			(State::Menu { menu, buttons }, Command::FinishMenu) => {
				println!(
					"menu {menu:?}, {}",
					buttons
						.iter()
						.map(|button| dcb_util::DisplayWrapper::new(move |f| write!(
							f,
							"\"{}\"",
							button.as_str().escape_debug()
						)))
						.format(", ")
				);

				*self = State::Start;
			},
			(State::Menu { menu, buttons }, Command::AddMenuOption { button }) => {
				anyhow::ensure!(
					menu.supports_button(button),
					"Menu {:?} doesn't support button \"{}\"",
					menu,
					button.as_str()
				);

				buttons.push(button)
			},
			(_, Command::FinishMenu) => anyhow::bail!("Can only call `finish_menu` when mid-menu"),
			(_, Command::AddMenuOption { .. }) => anyhow::bail!("Can only call `add_menu_option` when mid-menu"),

			(State::Menu { .. }, command) => anyhow::bail!("Cannot execute command {:?} mid-menu", command),
		}
		Ok(())
	}

	/// Drops this state
	pub fn finish(self) -> Result<(), anyhow::Error> {
		match self {
			State::Start => Ok(()),
			State::Menu { .. } => anyhow::bail!("Must call `finish_menu` to finish menu"),
		}
	}
}

/// Menu
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Menu {
	Three,
	Five,
}

impl Menu {
	/// Returns if a button may be used in this menu
	pub fn supports_button(self, button: MenuButton) -> bool {
		use MenuButton::*;
		match self {
			Self::Three => matches!(button, Talk | Battle | DeckData | Save | Yes | No | Cards | Partner),
			Self::Five => matches!(
				button,
				PlayerRoom |
					Menu | BattleCafe | BattleArena |
					ExtraArena | BeetArena |
					HauntedArena | FusionShop |
					Yes | No
			),
		}
	}
}

/// Menu buttons
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum MenuButton {
	PlayerRoom   = 0x0,
	Menu         = 0x1,
	BattleCafe   = 0x2,
	BattleArena  = 0x3,
	ExtraArena   = 0x4,
	BeetArena    = 0x5,
	HauntedArena = 0x6,
	FusionShop   = 0x7,
	Yes          = 0x8,
	No           = 0x9,
	Talk         = 0x0c,
	Battle       = 0x0d,
	DeckData     = 0x0e,
	Save         = 0x0f,
	Cards        = 0x12,
	Partner      = 0x13,
}

impl MenuButton {
	fn parse(value: u16) -> Option<Self> {
		let button = match value {
			0x0 => Self::PlayerRoom,
			0x1 => Self::Menu,
			0x2 => Self::BattleCafe,
			0x3 => Self::BattleArena,
			0x4 => Self::ExtraArena,
			0x5 => Self::BeetArena,
			0x6 => Self::HauntedArena,
			0x7 => Self::FusionShop,
			0x8 => Self::Yes,
			0x9 => Self::No,
			0x0c => Self::Talk,
			0x0d => Self::Battle,
			0x0e => Self::DeckData,
			0x0f => Self::Save,
			0x10 => Self::Yes,
			0x11 => Self::No,
			0x12 => Self::Cards,
			0x13 => Self::Partner,
			_ => return None,
		};
		Some(button)
	}
}

impl MenuButton {
	/// Returns a string representing this button
	pub fn as_str(self) -> &'static str {
		match self {
			Self::PlayerRoom => "Player's room",
			Self::Menu => "Menu",
			Self::BattleCafe => "Battle Cafe",
			Self::BattleArena => "Battle Arena",
			Self::ExtraArena => "Extra Arena",
			Self::BeetArena => "Beet Arena",
			Self::HauntedArena => "Haunted Arena",
			Self::FusionShop => "Fusion shop",
			Self::Yes => "Yes",
			Self::No => "No",
			Self::Talk => "Talk",
			Self::Battle => "Battle",
			Self::DeckData => "DeckData",
			Self::Save => "Save",
			Self::Cards => "Cards",
			Self::Partner => "Partner",
		}
	}
}

/// Command
#[derive(PartialEq, Clone, Debug)]
pub enum Command<'a> {
	/// Display buffer
	DisplayBuffer,

	/// Wait for input
	WaitInput,

	/// New screen
	NewScreen,

	/// Finish menu
	FinishMenu,

	/// Set value
	SetValue { var: u8, value0: u32, value1: u32 },

	/// Unknown07
	Unknown07 {
		value0: u8,
		kind:   u8,
		value1: u32,
		value2: u32,
	},

	/// Choice jump
	ChoiceJump {
		value0: u8,
		kind:   u8,
		value1: u32,
		value2: u32,
	},

	/// Jump
	Jump { value: u8, kind: u8, addr: u32 },

	/// Unknown 0a
	Unknown0a { value: u8, kind: u8 },

	/// Open menu
	OpenMenu { menu: Menu },

	/// Add menu option
	AddMenuOption { button: MenuButton },

	/// Display scene
	DisplayScene { value0: u8, value1: u16 },

	/// Set buffer
	SetBuffer { kind: u8, bytes: &'a [u8] },

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
			[0x0a, 0x0, 0x4, 0x0] => Self::DisplayBuffer,
			[0x0a, 0x0, 0x5, 0x0] => Self::WaitInput,
			[0x0a, 0x0, 0x6, 0x0] => Self::NewScreen,
			[0x0a, 0x0, 0x1, 0x0] => Self::FinishMenu,
			[0x0a, 0x0, value, kind] => Self::Unknown0a { value, kind },

			// Set variable
			[0x07, 0x0, var, 0x0] => {
				let value0 = LittleEndian::read_u32(slice.get(0x4..0x8)?);
				let value1 = LittleEndian::read_u32(slice.get(0x8..0xc)?);

				Self::SetValue { var, value0, value1 }
			},
			[0x07, 0x0, value0, kind] => {
				let value1 = LittleEndian::read_u32(slice.get(0x4..0x8)?);
				let value2 = LittleEndian::read_u32(slice.get(0x8..0xc)?);

				Self::Unknown07 {
					value0,
					kind,
					value1,
					value2,
				}
			},

			// Choice jump
			[0x09, 0x0, value0, kind] => {
				let value1 = LittleEndian::read_u32(slice.get(0x4..0x8)?);
				let value2 = LittleEndian::read_u32(slice.get(0x8..0xc)?);

				// value1: 0x3 0x5
				// kind: 0x0 0x1

				// value1: If 0x3, then buttons work normally
				// value1: If 0x1, then buttons work reverse
				// value1: If 0x5, they both choose "No"
				// value1: If 0x7, they both choose "Yes"

				// value2: If 0x0, they both choose "No"

				Self::ChoiceJump {
					value0,
					kind,
					value1,
					value2,
				}
			},

			// Jump?
			[0x05, 0x0, value, kind] => {
				let addr = LittleEndian::read_u32(slice.get(0x4..0x8)?);

				Self::Jump { value, kind, addr }
			},

			// Open menu
			[0x0b, 0x0, 0x0, 0x0] => {
				if slice.get(0x4..0x6)? != [0x0, 0x0] {
					return None;
				}
				let value = LittleEndian::read_u16(slice.get(0x6..0x8)?);

				// value: 0x61 0x78
				let menu = match value {
					0x61 => Menu::Three,
					0x78 => Menu::Five,
					_ => return None,
				};

				Self::OpenMenu { menu }
			},
			[0x0b, 0x0, 0x1, 0x0] => {
				if slice.get(0x4..0x6)? != [0x0, 0x0] {
					return None;
				}
				let value = LittleEndian::read_u16(slice.get(0x6..0x8)?);

				let button = MenuButton::parse(value)?;

				Self::AddMenuOption { button }
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
			Command::NewScreen => 4,
			Command::FinishMenu => 4,
			Command::SetValue { .. } => 0xc,
			Command::Unknown07 { .. } => 0xc,
			Command::ChoiceJump { .. } => 0xc,
			Command::Jump { .. } => 8,
			Command::Unknown0a { .. } => 4,
			Command::OpenMenu { .. } => 8,
			Command::AddMenuOption { .. } => 8,
			Command::DisplayScene { .. } => 8,
			Command::SetBuffer { bytes, .. } => {
				let len = bytes.len() + 2;
				4 + len + (4 - len % 4)
			},
			Command::SetBrightness { .. } => 16,
		}
	}
}
