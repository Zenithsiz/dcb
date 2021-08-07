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
	map_first_last,
	generic_associated_types
)]

// Modules
mod cli;

// Imports
use anyhow::Context;
use cli::CliData;
use dcb_msd::{ComboBox, ComboBoxButton, Inst};
use itertools::Itertools;
use std::{
	collections::{BTreeMap, HashMap},
	convert::TryInto,
	fs, mem,
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
			match Inst::decode(it.as_slice()) {
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

	// Get all variable names
	let vars: Result<_, anyhow::Error> = try {
		let known_vars_file_path = cli_data.input_file.with_file_name("msd.vars");
		let known_vars_file = std::fs::File::open(known_vars_file_path).context("Unable to open vars file")?;
		serde_yaml::from_reader::<_, HashMap<u16, String>>(known_vars_file).context("Unable to parse vars file")?
	};
	let vars = match vars {
		Ok(vars) => vars,
		Err(err) => {
			log::warn!("Unable to load variables: {err:?}");
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

		let mut stdout = std::io::stdout();
		let ctx = state.display_ctx(&labels, &vars);
		inst.display(&mut stdout, &ctx).context("Unable to display")?;
		mem::drop(ctx);

		println!();

		/*
		let bytes = &contents[(pos as usize)..((pos as usize) + inst.size())];
		print!(
			"[0x{}] ",
			bytes.iter().format_with("", |value, f| f(&format_args!("{value:02x}")))
		);
		*/

		state
			.apply(inst)
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
	/// Applies `inst` to this state machine
	pub fn apply(&mut self, inst: Inst) -> Result<(), anyhow::Error> {
		match (&mut *self, inst) {
			(Self::Start, Inst::OpenComboBox { combo_box: menu }) => {
				*self = Self::Menu { menu, buttons: vec![] };
			},
			(Self::Menu { .. }, Inst::ComboBoxAwait) => {
				*self = Self::Start;
			},
			(Self::Menu { menu, buttons }, Inst::AddComboBoxButton { value }) => {
				let button = menu.parse_button(value).context("Menu doesn't support button")?;
				buttons.push(button);
			},
			(_, Inst::ComboBoxAwait) => anyhow::bail!("Can only call `finish_menu` when mid-menu"),
			(_, Inst::AddComboBoxButton { .. }) => anyhow::bail!("Can only call `add_menu_option` when mid-menu"),

			(Self::Menu { .. }, inst) => anyhow::bail!("Cannot execute instruction {:?} mid-menu", inst),
			_ => (),
		}
		Ok(())
	}

	/// Drops this state
	pub fn finish(self) -> Result<(), anyhow::Error> {
		match self {
			Self::Start => Ok(()),
			Self::Menu { .. } => anyhow::bail!("Must call `finish_menu` to finish menu"),
		}
	}

	/// Returns the display context for this state
	pub fn display_ctx<'a>(
		&'a self, labels: &'a HashMap<u32, String>, vars: &'a HashMap<u16, String>,
	) -> impl dcb_msd::inst::DisplayCtx + 'a {
		DisplayCtx {
			state: self,
			labels,
			vars,
		}
	}
}


/// Display context
struct DisplayCtx<'a> {
	/// State
	state: &'a State,

	/// Labels
	labels: &'a HashMap<u32, String>,

	/// Variables
	vars: &'a HashMap<u16, String>,
}

impl<'a> dcb_msd::inst::DisplayCtx for DisplayCtx<'a> {
	type PosLabel<'b> = &'b str;
	type VarLabel<'b> = &'b str;

	fn cur_combo_box(&self) -> Option<ComboBox> {
		match self.state {
			State::Start => None,
			State::Menu { menu, .. } => Some(*menu),
		}
	}

	fn pos_label(&self, pos: u32) -> Option<Self::PosLabel<'_>> {
		self.labels.get(&pos).map(String::as_str)
	}

	fn var_label(&self, var: u16) -> Option<Self::VarLabel<'_>> {
		self.vars.get(&var).map(String::as_str)
	}
}
