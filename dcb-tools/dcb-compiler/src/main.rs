//! Compiler

#![feature(
	box_syntax,
	backtrace,
	panic_info_message,
	array_chunks,
	format_args_capture,
	bindings_after_at,
	unwrap_infallible
)]

// Modules
mod cli;

// Imports
use anyhow::Context;
use dcb_bytes::Bytes;
use dcb_exe::{
	inst::{
		parse::{line::InstParser, LineArg},
		Inst, Label, LabelName,
	},
	Data, Pos,
};
use dcb_util::AsciiStrArr;
use std::{
	collections::{BTreeMap, HashMap},
	convert::TryInto,
	fs,
	io::{BufReader, Seek, SeekFrom, Write},
};

fn main() -> Result<(), anyhow::Error> {
	// Initialize the logger
	simplelog::TermLogger::init(log::LevelFilter::Info, simplelog::Config::default(), simplelog::TerminalMode::Stderr)
		.expect("Unable to initialize logger");

	// Get all data from cli
	let cli = cli::CliData::new();

	// Get the header
	let header_file = fs::File::open(&cli.header_path).with_context(|| format!("Unable to open header file {}", cli.header_path.display()))?;
	let header: Header = serde_yaml::from_reader(header_file).context("Unable to read header file")?;

	// Open the input and output file
	let input_file = fs::File::open(&cli.input_path).context("Unable to open input file")?;
	let input_file = BufReader::new(input_file);
	let mut output_file = fs::File::create(&cli.output_file_path).context("Unable to open output file")?;

	// Read the input
	let lines = InstParser::new(input_file)
		.zip(0..)
		.map(|(res, n)| res.map(|line| (n, line)).map_err(|err| (n, anyhow::Error::from(err))));
	let mut cur_pos = header.start_pos;
	let res = itertools::process_results(lines, |lines| {
		let mut labels_by_name = HashMap::new();
		let mut labels_by_pos = BTreeMap::<Pos, Label>::new();

		let mut insts = BTreeMap::new();

		for (n, line) in lines {
			if let Some(label) = line.label {
				// Convert the label to ours
				let label = match label.name.starts_with('.') {
					// It's local
					true => {
						// Get the previous global label
						let prev_label_name = labels_by_pos
							.range(..=cur_pos)
							.filter_map(|(_, label)| label.as_global())
							.next_back()
							.ok_or_else(|| (n, anyhow::anyhow!("Cannot define a local label before any global labels")))?;

						// Then insert it
						let mut name = label.name;
						name.insert_str(0, prev_label_name);

						Label::Local { name: LabelName(name) }
					},
					// It's global
					false => Label::Global { name: LabelName(label.name) },
				};


				// Insert the label
				let name = label.name().clone();
				assert!(labels_by_pos.insert(cur_pos, label.clone()).is_none());
				assert!(labels_by_name.insert(name, cur_pos).is_none());
			}
			if let Some(mut inst) = line.inst {
				// Modify any local labels
				for arg in &mut inst.args {
					if let LineArg::Label(name) | LineArg::LabelOffset { label: name, .. } = arg {
						// If the label isn't local, continue
						if !name.starts_with('.') {
							continue;
						}

						// Get the previous global label
						let prev_label_name = labels_by_pos
							.range(..=cur_pos)
							.filter_map(|(_, label)| label.as_global())
							.next_back()
							.ok_or_else(|| (n, anyhow::anyhow!("Cannot define a local label before any global labels")))?;

						// Then insert it
						name.insert_str(0, prev_label_name);
					}
				}

				let inst_size = Inst::size_from_parsed(&inst, cur_pos).map_err(|_| (n, anyhow::anyhow!("Unable to compile instruction")))?;

				assert!(insts.insert(cur_pos, (n, inst)).is_none());

				cur_pos += inst_size;
			}
		}

		Ok((labels_by_name, labels_by_pos, insts))
	});
	let (mut labels_by_name, _labels_by_pos, insts) = match res {
		Ok(Ok(v)) => v,
		Ok(Err((n, err))) | Err((n, err)) => return Err(err).context(format!("Unable to parse line {}", n + 1)),
	};

	// Read all foreign data as labels.
	let foreign_data_file = std::fs::File::open("resources/foreign_data.yaml").context("Unable to open foreign data file")?;
	let foreign_data: Vec<Data> = serde_yaml::from_reader(foreign_data_file).context("Unable to read foreign data file")?;
	for data in foreign_data {
		let (pos, name) = data.into_label();
		labels_by_name.insert(name, pos);
	}

	// Seek to the start of the instructions
	output_file
		.seek(SeekFrom::Start(0x800))
		.context("Unable to seek stream to beginning of instructions")?;

	// For each instruction, pack it and output it to the file
	for (&pos, (n, inst)) in &insts {
		let inst =
			Inst::from_parsed(inst, pos, &labels_by_name).with_context(|| format!("Unable to compile instruction at {} in line {}", pos, n + 1))?;

		inst.write(&mut output_file).context("Unable to write to file")?;
	}

	let size = output_file.stream_position().context("Unable to get stream position")? - 0x800;
	let size = size.try_into().context("Size was too large")?;

	// Go back and write the header
	let header = dcb_exe::Header {
		pc0: header.pc0,
		gp0: header.gp0,
		start_pos: header.start_pos,
		size,
		unknown20: header.unknown20,
		unknown24: header.unknown24,
		memfill_start: header.memfill_start,
		memfill_size: header.memfill_size,
		initial_sp_base: header.initial_sp_base,
		initial_sp_offset: header.initial_sp_offset,
		marker: header.marker,
	};
	output_file.seek(SeekFrom::Start(0)).context("Unable to seek stream to beginning")?;
	let mut header_bytes = [0; 0x800];
	header.to_bytes(&mut header_bytes).into_ok();
	output_file.write_all(&header_bytes).context("Unable to write header to output file")?;

	Ok(())
}

/// Header
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[derive(serde::Deserialize)]
pub struct Header {
	/// Initial program counter
	pub pc0: u32,

	/// Initial global pointer
	pub gp0: u32,

	/// Starting position, in memory, of the executable.
	pub start_pos: Pos,

	/// Unknown at `0x20`
	pub unknown20: u32,

	/// Unknown at `0x24`
	pub unknown24: u32,

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
