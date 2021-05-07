//! Assembler

#![feature(unwrap_infallible, format_args_capture, try_blocks, hash_raw_entry, bool_to_option)]

// Modules
mod cli;

// Imports
use anyhow::Context;
use dcb_bytes::Bytes;
use dcb_exe::{
	inst::{
		parse::{Line, LineInst},
		Inst, InstSize, Label, Parsable, ParseCtx,
	},
	Data, Pos,
};
use dcb_util::{AsciiStrArr, BTreeMapVector};
use std::{
	collections::{BTreeMap, HashMap},
	convert::TryInto,
	fs,
	hash::{BuildHasher, Hash, Hasher},
	io::{BufRead, BufReader, Seek, SeekFrom, Write},
	rc::Rc,
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
	let cli::CliData {
		input_path,
		header_path,
		output_file_path,
	} = cli::CliData::new();

	// Get the header
	let header: Header = dcb_util::parse_from_file(&header_path, serde_yaml::from_reader)
		.with_context(|| format!("Unable to read header file {header_path:?}"))?;

	// Open the input and output file
	let input_file = fs::File::open(&input_path)
		.map(BufReader::new)
		.with_context(|| format!("Unable to open input file {input_path:?}"))?;
	let mut output_file = fs::File::create(&output_file_path)
		.with_context(|| format!("Unable to open output file {output_file_path:?}"))?;

	// All labels and instructions
	let mut labels_by_name = HashMap::<Rc<Label>, Pos>::new();
	let mut labels_by_pos = BTreeMapVector::<Pos, Rc<Label>>::new();
	let mut insts = BTreeMap::<usize, (bool, LineInst)>::new();

	// Read all foreign data as labels.
	let foreign_data_file_path = "resources/foreign_data.yaml";
	let foreign_data: Vec<Data> = dcb_util::parse_from_file(foreign_data_file_path, serde_yaml::from_reader)
		.with_context(|| format!("Unable to read foreign data file {foreign_data_file_path:?}"))?;
	for data in foreign_data {
		let (pos, label) = data.into_label();
		let label = Rc::new(label);

		labels_by_name.insert(Rc::clone(&label), pos);
		labels_by_pos.insert(pos, label)
	}

	// Read all lines within the input
	let mut cur_pos = header.start_pos;
	for (line_idx, line) in input_file.lines().enumerate() {
		let line_idx = line_idx + 1;
		let line = line.with_context(|| format!("Unable to read line {line_idx}"))?;
		let line = Line::parse(&line).with_context(|| format!("Unable to parse line {line_idx}"))?;

		// Add every label we get
		for label in line.labels {
			/// Helper function to add a label
			fn add_label(
				mut label_name: String, cur_pos: Pos, labels_by_name: &mut HashMap<Rc<Label>, Pos>,
				labels_by_pos: &mut BTreeMapVector<Pos, Rc<Label>>,
			) -> Result<(), anyhow::Error> {
				// If it starts with `.`, convert it to a global label
				if label_name.starts_with('.') {
					let last_label = labels_by_pos
						.range(..=cur_pos)
						.filter(|(_, label)| label.is_global())
						.next_back()
						.with_context(|| format!("Cannot use a local label {label_name:?} before any global labels"))?
						.1;

					label_name.insert_str(0, last_label);
				}

				// Then put it in an rc
				let label_name = Rc::new(Label::new(label_name));

				// Then try to insert it
				if let Some(label) = labels_by_name.insert(Rc::clone(&label_name), cur_pos) {
					anyhow::bail!("Cannot add duplicate label {:?}", label);
				}
				labels_by_pos.insert(cur_pos, label_name);

				Ok(())
			}

			add_label(label.name, cur_pos, &mut labels_by_name, &mut labels_by_pos)
				.with_context(|| format!("Unable to add label in line {line_idx}"))?;
		}

		// If this line has an instruction, add it
		if let Some(mut inst) = line.inst {
			// Modify any local labels within the instruction to be global
			for arg in &mut inst.args {
				// Try to get it as a label
				let label_name = match arg.as_expr_mut().and_then(|expr| expr.as_label_mut()) {
					Some((label, ..)) => label,
					None => continue,
				};

				// If it doesn't start with `.`, ignore it
				if !label_name.starts_with('.') {
					continue;
				}

				// Then get the previous global label
				let prev_label = labels_by_pos
					.range(..=cur_pos)
					.filter(|(_, label)| label.is_global())
					.next_back()
					.with_context(|| format!("Cannot use a local label {label_name:?} before any global labels"))?
					.1;

				label_name.insert_str(0, prev_label);
			}

			// Then insert the instruction, get it's size and update our current position
			// TODO: Better solution than assembling the instruction with a dummy context.
			let inst_size = Inst::parse(&inst.mnemonic, &inst.args, &DummyCtx { pos: cur_pos })
				.with_context(|| format!("Unable to compile instruction in line {line_idx}"))?
				.size();

			assert!(insts.insert(line_idx, (line.branch_delay, inst)).is_none());
			cur_pos += inst_size;
		}
	}

	// Seek to the start of the instructions
	output_file
		.seek(SeekFrom::Start(0x800))
		.context("Unable to seek stream to beginning of instructions")?;

	// For each instruction, pack it and output it to the file
	let mut cur_pos = header.start_pos;
	let mut last_inst = None;
	for (&line_idx, &(branch_delay, ref inst)) in &insts {
		// Create the context
		let ctx = Ctx {
			pos:            cur_pos,
			labels_by_name: &labels_by_name,
		};

		// Make sure this instruction has an branch delay marker is the previous instruction
		// has a jump
		anyhow::ensure!(
			self::implies(branch_delay, || last_inst.as_ref().map_or(false, Inst::may_jump)),
			"{}: Branch delay marker must be used only when the previous instruction may jump",
			line_idx
		);
		anyhow::ensure!(
			self::implies(!branch_delay, || !last_inst.as_ref().map_or(false, Inst::may_jump)),
			"{}: Branch delay marker is required when the previous instruction may jump",
			line_idx
		);

		let inst = Inst::parse(&inst.mnemonic, &inst.args, &ctx)
			.with_context(|| format!("{line_idx}: Unable to compile instruction for {}", cur_pos))?;

		// If we got a pseudo instruction larger than 1 basic instruction after a jump, return Err
		anyhow::ensure!(
			self::implies(branch_delay, || inst.size() == 4),
			"{}: Branch delays cannot use pseudo instructions larger than 4 bytes",
			line_idx
		);

		inst.write(&mut output_file)
			.context("Unable to write instruction to file")?;
		last_inst = Some(inst);
		cur_pos += inst.size();
	}

	let size = output_file.stream_position().context("Unable to get stream position")? - 0x800;
	let size = size.try_into().context("Size was too large")?;

	// Go back and write the header
	let header = dcb_exe::Header {
		pc0: self::pos_by_label_name(&labels_by_name, "start")
			.context("No `start` label found")?
			.0,
		gp0: header.gp0,
		start_pos: header.start_pos,
		size,
		memfill_start: header.memfill_start,
		memfill_size: header.memfill_size,
		initial_sp_base: header.initial_sp_base,
		initial_sp_offset: header.initial_sp_offset,
		marker: header.marker,
	};
	output_file
		.seek(SeekFrom::Start(0))
		.context("Unable to seek stream to beginning")?;
	let mut header_bytes = [0; 0x800];
	header.to_bytes(&mut header_bytes).into_ok();
	output_file
		.write_all(&header_bytes)
		.context("Unable to write header to output file")?;

	Ok(())
}

/// Dummy parsing context to get instruction size.
// TODO: Find better solution than this?
struct DummyCtx {
	/// Current position
	pos: Pos,
}

impl ParseCtx<'_> for DummyCtx {
	fn cur_pos(&self) -> Pos {
		self.pos
	}

	fn label_pos(&self, _label: &str) -> Option<Pos> {
		Some(self.pos)
	}
}

/// Parsing context
struct Ctx<'a> {
	/// Current position
	pos: Pos,

	/// All labels by name
	labels_by_name: &'a HashMap<Rc<Label>, Pos>,
}

impl<'a> ParseCtx<'a> for Ctx<'a> {
	fn cur_pos(&self) -> Pos {
		self.pos
	}

	fn label_pos(&self, label: &str) -> Option<Pos> {
		self::pos_by_label_name(&self.labels_by_name, label)
	}
}

/// Helper function to retrieve a position by it's label name
pub fn pos_by_label_name(labels_by_name: &HashMap<Rc<Label>, Pos>, label: &str) -> Option<Pos> {
	let mut state = labels_by_name.hasher().build_hasher();
	label.hash(&mut state);
	let hash = state.finish();
	labels_by_name
		.raw_entry()
		.from_hash(hash, |lhs| lhs.as_str() == label)
		.map(|(_, &pos)| pos)
}

/// Header
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[derive(serde::Deserialize)]
struct Header {
	/// Initial global pointer
	pub gp0: u32,

	/// Starting position, in memory, of the executable.
	pub start_pos: Pos,

	/// Where to start mem filling
	pub memfill_start: u32,

	/// Size to mem fill
	pub memfill_size: u32,

	/// Initial stack pointer
	pub initial_sp_base: u32,

	/// Offset from initial stack pointer
	pub initial_sp_offset: u32,

	/// Executable region marker
	pub marker: AsciiStrArr<0x7b3>,
}

/// Returns the logical implication of a boolean and a predicate
pub fn implies(lhs: bool, rhs: impl FnOnce() -> bool) -> bool {
	lhs.then(rhs).unwrap_or(true)
}
