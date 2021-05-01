//! Decompiler

#![feature(try_blocks, format_args_capture, iter_map_while, box_syntax)]

// Modules
mod cli;

// Imports
use anyhow::Context;
use dcb_exe::{
	inst::{
		exec::{ExecError, ExecState, SysCallback},
		Register,
	},
	Pos,
};
use std::{cell::RefCell, collections::HashMap, fs};

fn main() -> Result<(), anyhow::Error> {
	// Initialize the logger
	simplelog::TermLogger::init(
		log::LevelFilter::Info,
		simplelog::Config::default(),
		simplelog::TerminalMode::Stderr,
	)
	.expect("Unable to initialize logger");

	// Get all data from cli
	let cli = cli::CliData::new();

	// Open the input file
	let input_bytes = fs::read(&cli.input_path).context("Unable to read input file")?;

	// Then put them at `0x10000`
	let mut memory = vec![0; 0x10000];
	memory.extend_from_slice(&input_bytes[0x800..]);

	// Create the executor
	let mut exec_state = ExecState::new(memory.into(), Pos(0x80010000));

	// Setup syscalls
	let should_stop = RefCell::new(false);
	let sys0 = |_: &mut ExecState| {
		*should_stop.borrow_mut() = true;
		Ok(())
	};
	let sys1 = |state: &mut ExecState| {
		// Print whatever string is in `$v0`
		let ptr = Pos(state[Register::V0]);

		for n in 0u32.. {
			match state.read_byte(ptr + n)? {
				0 => break,
				b => print!("{}", char::from(b)),
			}
		}

		Ok(())
	};
	let mut syscalls: HashMap<u32, Box<SysCallback>> =
		vec![(0, box_fn_mut(sys0)), (1, box_fn_mut(sys1))].into_iter().collect();

	while !*should_stop.borrow() {
		exec_state
			.exec(&mut syscalls)
			.with_context(|| format!("Failed to execute at {}", exec_state.pc()))?;
	}


	Ok(())
}

fn box_fn_mut<'a>(
	f: impl FnMut(&mut ExecState) -> Result<(), ExecError> + 'a,
) -> Box<dyn FnMut(&mut ExecState) -> Result<(), ExecError> + 'a> {
	box f
}
