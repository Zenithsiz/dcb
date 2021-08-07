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
	cow_is_borrowed,
	map_first_last
)]

// Modules
mod cli;

// Imports
use anyhow::Context;
use cli::CliData;
use dcb_msd::{ComboBox, ComboBoxButton, Inst};
use encoding_rs::SHIFT_JIS;
use itertools::Itertools;
use std::{
	collections::{BTreeMap, HashMap},
	convert::TryInto,
	fs,
};


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

	// Parse all instructions
	let insts = contents
		.iter()
		.batching(|it| {
			let pos = it.as_slice().as_ptr() as usize - contents.as_slice().as_ptr() as usize;
			let pos = match pos.try_into() {
				Ok(pos) => pos,
				Err(_) => return Some(Err(anyhow::anyhow!("Position {:#x} didn't fit into a `u32`", pos))),
			};
			match Inst::parse(it.as_slice()) {
				Some(inst) => {
					it.advance_by(inst.size())
						.expect("Iterator had less elements than size of instruction");
					Some(Ok((pos, inst)))
				},
				None => match it.is_empty() {
					true => None,
					false => Some(Err(anyhow::anyhow!(
						"Unable to parse instruction at {:#010x}: {:?}",
						pos,
						&it.as_slice()[..4]
					))),
				},
			}
		})
		.collect::<Result<BTreeMap<_, _>, anyhow::Error>>()
		.context("Unable to parse instructions")?;

	log::info!("Found {} instructions", insts.len());

	// Get all value names
	let values: Result<_, anyhow::Error> = try {
		let known_values_file_path = cli_data.input_file.with_file_name("msd.values");
		let known_values_file = std::fs::File::open(known_values_file_path).context("Unable to open values file")?;
		serde_yaml::from_reader::<_, HashMap<u16, String>>(known_values_file).context("Unable to parse values file")?
	};
	let values = match values {
		Ok(values) => values,
		Err(err) => {
			log::warn!("Unable to load values: {err:?}");
			HashMap::new()
		},
	};


	// Get all labels
	let labels: Result<_, anyhow::Error> = try {
		let known_labels_file_path = format!("{}.labels", cli_data.input_file.display());
		let known_labels_file = std::fs::File::open(known_labels_file_path).context("Unable to open labels file")?;
		serde_yaml::from_reader::<_, HashMap<u32, String>>(known_labels_file).context("Unable to parse labels file")?
	};
	let mut labels = match labels {
		Ok(labels) => labels,
		Err(err) => {
			log::warn!("Unable to load labels: {err:?}");
			HashMap::new()
		},
	};

	let heuristic_labels = insts
		.iter()
		.filter_map(|(_pos, inst)| match *inst {
			Inst::Jump { addr, .. } if !labels.contains_key(&addr) => Some(addr),
			_ => None,
		})
		.unique()
		.sorted()
		.enumerate()
		.map(|(idx, addr)| (addr, format!("jump_{idx}")));
	labels.extend(heuristic_labels);

	/*
	#[derive(Clone, Debug)]
	enum CallCond {
		None,
		VarEq { var: u16, value: u32 },
	}

	#[derive(Clone, Debug)]
	struct Block {
		start: u32,
		end:   u32,

		calls: BTreeMap<u32, (u32, CallCond)>,
	}

	let blocks = labels
		.keys()
		.map(|&pos| {
			let (end_pos, label_at_end, unconditional_jump) = commands
				.range(pos..)
				.tuple_windows()
				.find_map(|((&cur_pos, cur), (&next_pos, next))| {
					// If the first instruction is a jump, it's unconditional
					if pos == cur_pos {
						if let Command::Jump { addr, .. } = *cur {
							return Some((next_pos, labels.contains_key(&next_pos), Some((cur_pos, addr))));
						}
					}

					// Else check if the next instruction is a label
					if labels.contains_key(&next_pos) {
						// Note: Here the unconditional must be `None` or we would have
						//       returned on the previous iteration or above.
						return Some((next_pos, true, None));
					}

					// Else check for a non-test + jump combo.
					match (cur, next) {
						// Ignore test-jumps
						(Command::Test { .. }, Command::Jump { .. }) => None,

						// Else a non-test jump should be unconditional
						// Note: `unconditional_jump` should be false here always I believe, else it's dead code?
						(_, Command::Jump { .. }) => Some((next_pos + next.size() as u32, false, match *cur {
							Command::Jump { addr, .. } => Some((cur_pos, addr)),
							_ => None,
						})),
						_ => None,
					}
				})
				.unwrap_or_else(|| (commands.last_key_value().map_or(0, |(&pos, _)| pos), false, None));

			let mut calls = commands
				.range(pos..end_pos)
				.tuple_windows()
				.filter_map(|((&cur_pos, cur), (&next_pos, next))| match (cur, next) {
					(Command::Test { var, value2: value, .. }, Command::Jump { addr, .. }) => Some((
						next_pos,
						(*addr, CallCond::VarEq {
							var:   *var,
							value: *value,
						}),
					)),

					// Diverging calls
					(_, Command::Jump { addr, .. }) => Some((cur_pos, (*addr, CallCond::None))),
					_ => None,
				})
				.collect::<BTreeMap<_, _>>();

			// Check if we need to add any extra calls
			match (label_at_end, unconditional_jump) {
				// If we ended on a label without diverging, add a call the label
				(true, None) => calls.insert(end_pos, (end_pos, CallCond::None)).void(),

				// If we ended by diverging, insert a call to it.
				(_, Some((pos, addr))) => calls.insert(pos, (addr, CallCond::None)).void(),

				// Else no extra calls
				_ => (),
			}

			(pos, Block {
				start: pos,
				end: end_pos,
				calls,
			})
		})
		.collect::<BTreeMap<u32, Block>>();

	let dot_file_path = format!("{}.dot", cli_data.input_file.display());
	let mut dot_file = std::fs::File::create(dot_file_path).context("Unable to create dot file")?;

	writeln!(dot_file, "digraph \"G\" {{").context("Unable to write to dot file")?;
	for block in blocks.values() {
		let block_label = labels.get(&block.start).expect("Block had no label");
		writeln!(dot_file, "\t{block_label};").context("Unable to write to dot file")?;
		// TODO: Move unique from here to `calls` maybe?
		//       Might not work with two separate values going to the same address.
		for (call_pos, cond) in block.calls.values().unique_by(|(call_pos, _)| call_pos) {
			let call_label = match labels.get(call_pos) {
				Some(label) => label.to_owned(),
				None => format!("\"{call_pos:#x}\""),
			};
			let cond_label = match cond {
				CallCond::None => "Otherwise".to_owned(),
				CallCond::VarEq { var, value } => {
					let var_label = match values.get(var) {
						Some(label) => label.to_owned(),
						None => format!("{var:#x}"),
					};

					format!("{var_label} == {value}")
				},
			};
			writeln!(
				dot_file,
				"\t{block_label} -> {call_label} [label = \"{}\"];",
				cond_label.escape_debug()
			)
			.context("Unable to write to dot file")?;
		}
	}
	writeln!(dot_file, "}}").context("Unable to write to dot file")?;
	*/

	let mut state = State::Start;
	for (pos, inst) in insts {
		if let Some(label) = labels.get(&pos) {
			println!("{label}:");
		};


		print!("{pos:#010x}: ");

		/*
		let bytes = &contents[(pos as usize)..((pos as usize) + inst.size())];
		print!(
			"[0x{}] ",
			bytes.iter().format_with("", |value, f| f(&format_args!("{value:02x}")))
		);
		*/

		state
			.parse_next(&labels, &values, inst)
			.with_context(|| format!("Unable to parse instruction at {pos:#010x} in current context"))?;
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
		menu: ComboBox,

		/// All buttons
		buttons: Vec<ComboBoxButton>,
	},
}

impl State {
	/// Parses the next instruction
	pub fn parse_next(
		&mut self, labels: &HashMap<u32, String>, values: &HashMap<u16, String>, inst: Inst,
	) -> Result<(), anyhow::Error> {
		match (&mut *self, inst) {
			(State::Start, Inst::DisplayTextBuffer) => println!("display_buffer"),
			(State::Start, Inst::WaitInput) => println!("wait_input"),
			(State::Start, Inst::EmptyTextBox) => println!("clear_screen"),
			(State::Start, Inst::SetBgBattleCafe) => println!("display_battle_cafe"),
			(State::Start, Inst::OpenScreen(screen)) => println!("open_screen \"{}\"", screen.as_str().escape_debug()),
			(State::Start, Inst::SetBgBattleArena) => println!("display_battle_arena"),
			(State::Start, Inst::DisplayCenterTextBox) => println!("display_text_box"),
			(State::Start, Inst::ChangeVar { var, op, value: value1 }) => {
				let value = match values.get(&var) {
					Some(value) => value.to_owned(),
					None => format!("{var:#x}"),
				};

				let op = match op {
					0 => "set",
					1 => "add",
					6 => "other",
					_ => unreachable!(),
				};

				println!("set_value {value}, {op}, {value1:#x}");
			},
			(
				State::Start,
				Inst::Test {
					var,
					op: value1,
					value: value2,
				},
			) => match values.get(&var) {
				Some(value) => println!("test {value}, {value1:#x}, {value2:#x}"),
				None => println!("test {var:#x}, {value1:#x}, {value2:#x}"),
			},

			(State::Start, Inst::Jump { var, addr }) => {
				let label = match labels.get(&addr) {
					Some(label) => label.to_owned(),
					None => format!("{addr:#010x}"),
				};

				println!("jump {var:#x}, {label}")
			},
			(State::Start, Inst::Unknown0a { value }) => println!("unknown_0a {value:#x}"),
			(State::Start, Inst::OpenComboBox { combo_box: menu }) => {
				*self = State::Menu { menu, buttons: vec![] };
				println!("open_menu {}", menu.as_str());
			},
			(State::Start, Inst::DisplayScene { value0, value1 }) => match (value0, value1) {
				(0x2, _) => println!("battle {value1:#x}"),

				_ => println!("display_scene {value0:#x}, {value1:#x}"),
			},
			(State::Start, Inst::SetBuffer { buffer: kind, bytes }) => {
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
				Inst::SetBrightness {
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
			(State::Menu { .. }, Inst::ComboBoxAwait) => {
				*self = State::Start;
				println!("finish_menu");
			},
			(State::Menu { menu, buttons }, Inst::AddComboBoxButton { value }) => {
				let button = menu.parse_button(value).context("Menu doesn't support button")?;

				buttons.push(button);

				println!("add_menu \"{}\"", button.as_str().escape_debug());
			},
			(_, Inst::ComboBoxAwait) => anyhow::bail!("Can only call `finish_menu` when mid-menu"),
			(_, Inst::AddComboBoxButton { .. }) => anyhow::bail!("Can only call `add_menu_option` when mid-menu"),

			(State::Menu { .. }, inst) => anyhow::bail!("Cannot execute instruction {:?} mid-menu", inst),
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
