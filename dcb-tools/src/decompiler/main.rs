//! Decompiler

#![feature(
	box_syntax,
	backtrace,
	panic_info_message,
	unsafe_block_in_unsafe_fn,
	array_value_iter,
	array_chunks,
	format_args_capture,
	or_patterns,
	bindings_after_at
)]
// Lints
#![warn(clippy::restriction, clippy::pedantic, clippy::nursery)]
// Instead of `unwrap`, we must use `expect` and provide a reason
#![forbid(clippy::unwrap_used)]
// We must use `unsafe` in unsafe `fn`s and specify if the guarantee is
// made by the caller or by us.
#![forbid(unsafe_op_in_unsafe_fn)]
// We'll disable the ones we don't need
#![allow(clippy::blanket_clippy_restriction_lints)]
// Necessary items may be inlined using `LTO`, so we don't need to mark them as inline
#![allow(clippy::missing_inline_in_public_items)]
// We prefer tail returns where possible, as they help with code readability in most cases.
#![allow(clippy::implicit_return)]
// We're fine with shadowing, as long as the variable is used for the same purpose.
// Hence why `clippy::shadow_unrelated` isn't allowed.
#![allow(clippy::shadow_reuse, clippy::shadow_same)]
// We panic when we know it won't happen, or if it does happen, then a panic is the best option
#![allow(clippy::panic, clippy::expect_used, clippy::unreachable, clippy::todo)]
// We use `expect` even in functions that return a `Result` / `Option` if there is a logic error
#![allow(clippy::unwrap_in_result)]
// We find it more important to be able to copy paste literals such as `0xabcd1234` than
// being able to read them, which does not provide many benefits
#![allow(clippy::unreadable_literal, clippy::unseparated_literal_suffix)]
// We separate implementations per their functionality usually, such as constructors, getters, setters, and others.
#![allow(clippy::multiple_inherent_impl)]
// Many operations we need to repeat, and to keep symmetry
#![allow(clippy::identity_op)]
// We only introduce items before their first usage, which sometimes is half-way through the code.
// We make sure that we only use the item after introduced, however.
#![allow(clippy::items_after_statements)]
// Useful for when they either change a lot with new variants / data,
// or for symmetry purposes
#![allow(clippy::match_same_arms)]
// In this library we have very grain-level error types, each function
// will have it's own error type ideally, so any errors are explicit
// by the type, without needing a section for them
#![allow(clippy::missing_errors_doc)]
// Although we generally try to avoid this, this can happen due to our module organization.
// In the future, this lint should be removed globally and only enabled for modules which
// actually require the use of it.
#![allow(clippy::module_inception, clippy::module_name_repetitions)]
// We use integer arithmetic and operations with the correct intent
#![allow(clippy::integer_arithmetic, clippy::integer_division)]
// We prefer using match ergonomic where possible
#![allow(clippy::pattern_type_mismatch)]
// Sometimes the blocks make it easier to invert their order
#![allow(clippy::if_not_else)]
// This lint triggers when using `assert`s and `todo`s, which is unsuitable for this project
#![allow(clippy::panic_in_result_fn)]
// We want to print the resulting instructions to stdout in this binary.
#![allow(clippy::print_stdout)]
// Lint goes off when going byte by byte in binary, not useful
#![allow(clippy::large_digit_groups)]
// We don't put the final `else` if it's empty
#![allow(clippy::else_if_without_else)]
// We're usually fine with missing future variants
#![allow(clippy::wildcard_enum_match_arm)]

// Modules
mod cli;
#[path = "../logger.rs"]
mod logger;

// Imports
use std::collections::{HashMap, HashSet};

use anyhow::Context;
use byteorder::{ByteOrder, LittleEndian};
use dcb::{
	game::exe::{
		func::Funcs,
		instruction::{
			Directive,
			PseudoInstruction::{self, Nop},
			Raw, Register, SimpleInstruction,
		},
		Instruction, Pos,
	},
	GameFile,
};
use itertools::Itertools;
use ref_cast::RefCast;

#[allow(clippy::cognitive_complexity, clippy::too_many_lines)] // TODO: Refactor
fn main() -> Result<(), anyhow::Error> {
	// Initialize the logger and set the panic handler
	logger::init();

	// Get all data from cli
	let cli::CliData { game_file_path } = cli::CliData::new();

	// Open the game file
	let input_file = std::fs::File::open(&game_file_path).context("Unable to open input file")?;
	let mut game_file = GameFile::from_reader(input_file).context("Unable to parse input file as dcb")?;

	// Read the executable
	log::debug!("Deserializing executable");
	let exe = dcb::game::Exe::deserialize(&mut game_file).context("Unable to parse game executable")?;

	// Get all instructions
	log::debug!("Retrieving all instructions");
	let instructions: Vec<(Pos, Instruction)> = Instruction::new_iter(
		exe.data
			.array_chunks::<4>()
			.map(|bytes| LittleEndian::read_u32(bytes))
			.zip(0..)
			.map(|(word, offset)| Raw {
				repr: word,
				pos:  Pos(exe.header.dest + 4 * offset),
			}),
	)
	.collect();

	// Get all functions
	log::debug!("Retrieving all functions");
	let functions: Funcs<String> = Funcs::known()
		.into_string()
		.merge(Funcs::from_instructions(
			instructions.iter().map(|(pos, instruction)| (*pos, instruction)),
		))
		.collect();

	// All instruction offsets
	log::debug!("Retrieving all offsets");
	let offsets: HashSet<Pos> = instructions.iter().map(|(offset, _)| offset).copied().collect();

	// All data / string addresses
	log::debug!("Retrieving all data / strings addresses");
	let data_string_addresses: HashSet<Pos> = instructions
		.iter()
		.filter_map(|(_, instruction)| match instruction {
			Instruction::Pseudo(
				PseudoInstruction::La { target: offset, .. } |
				PseudoInstruction::Li32 { imm: offset, .. } |
				PseudoInstruction::LbImm { offset, .. } |
				PseudoInstruction::LbuImm { offset, .. } |
				PseudoInstruction::LhImm { offset, .. } |
				PseudoInstruction::LhuImm { offset, .. } |
				PseudoInstruction::LwlImm { offset, .. } |
				PseudoInstruction::LwImm { offset, .. } |
				PseudoInstruction::LwrImm { offset, .. } |
				PseudoInstruction::SbImm { offset, .. } |
				PseudoInstruction::ShImm { offset, .. } |
				PseudoInstruction::SwlImm { offset, .. } |
				PseudoInstruction::SwImm { offset, .. } |
				PseudoInstruction::SwrImm { offset, .. },
			) |
			Instruction::Directive(Directive::Dw(offset) | Directive::DwRepeated { value: offset, .. }) => Some(Pos(*offset)),
			_ => None,
		})
		.collect();

	// Get all local jumps
	log::debug!("Retrieving all local jumps");
	let locals_pos: HashMap<Pos, usize> = instructions
		.iter()
		.filter_map(|(_, instruction)| match *instruction {
			Instruction::Simple(
				SimpleInstruction::J { target } |
				SimpleInstruction::Beq { target, .. } |
				SimpleInstruction::Bne { target, .. } |
				SimpleInstruction::Bltz { target, .. } |
				SimpleInstruction::Bgez { target, .. } |
				SimpleInstruction::Bgtz { target, .. } |
				SimpleInstruction::Blez { target, .. } |
				SimpleInstruction::Bltzal { target, .. } |
				SimpleInstruction::Bgezal { target, .. },
			) |
			Instruction::Pseudo(
				PseudoInstruction::Beqz { target, .. } | PseudoInstruction::Bnez { target, .. } | PseudoInstruction::B { target },
			) => Some(target),
			_ => None,
		})
		.filter(|target| (Instruction::CODE_START..Instruction::CODE_END).contains(target) && offsets.contains(target))
		.unique()
		.zip(0..)
		.collect();

	// Get all strings
	log::debug!("Retrieving all strings");
	let strings_pos: HashMap<Pos, usize> = instructions
		.iter()
		.filter_map(|(cur_pos, instruction)| match instruction {
			Instruction::Directive(Directive::Ascii(_)) => Some(*cur_pos),
			_ => None,
		})
		.filter(|cur_pos| data_string_addresses.contains(cur_pos))
		.unique()
		.zip(0..)
		.collect();

	// Get all data
	log::debug!("Retrieving all data");
	let data_pos: HashMap<Pos, usize> = instructions
		.iter()
		.filter_map(|(cur_pos, instruction)| match instruction {
			Instruction::Directive(Directive::Dw(_) | Directive::DwRepeated { .. }) => Some(*cur_pos),
			_ => None,
		})
		.filter(|cur_pos| data_string_addresses.contains(cur_pos))
		.unique()
		.zip(0..)
		.collect();

	// Build the full instructions iterator
	let full_iter = functions
		.with_instructions(instructions.iter().map(|(pos, instruction)| (*pos, instruction)))
		.scan(None, |last_instruction, output @ (_, cur_instruction, _)| {
			Some((output, last_instruction.replace(cur_instruction)))
		});

	// Read all instructions
	let mut skipped_nops = 0;
	for ((cur_pos, instruction, cur_func), last_instruction) in full_iter {
		// Note: Required by `rust-analyzer` currently, it can't determine the type of `cur_func`.
		let cur_func: Option<&dcb::game::exe::Func<String>> = cur_func;

		// If both last and current instructions are nops, skip
		if let (Some(Instruction::Pseudo(Nop)), Instruction::Pseudo(Nop)) = (last_instruction, instruction) {
			skipped_nops += 1;
			continue;
		}

		// If we skipped any nops, output the number of skipped nops
		// TODO: Merge nops in `Pseudo` or something.
		if skipped_nops != 0 {
			println!("# + {skipped_nops} x nop");
			skipped_nops = 0;
		}

		// Check if we need to prefix
		match cur_func {
			Some(cur_func) if cur_func.start_pos == cur_pos => {
				println!("####################");
				println!("{}:", cur_func.name);
				println!("# {}\n#", cur_func.signature);
				for description in cur_func.desc.lines() {
					println!("# {}", description);
				}
			},
			_ => (),
		}
		if let Some(local_idx) = locals_pos.get(&cur_pos) {
			println!("\t.{local_idx}:");
		}
		if let Some(string_idx) = strings_pos.get(&cur_pos) {
			println!("\tstring_{string_idx}:");
		}
		if let Some(data_idx) = data_pos.get(&cur_pos) {
			println!("\tdata_{data_idx}:");
		}

		// Print the instruction
		print!("{cur_pos:#010x}: {instruction}");

		// Check if we should have any comments with this instruction
		// TODO: Add Pseudo jumps too
		match instruction {
			// If we have a jump, make a comment with it's target
			Instruction::Simple(
				SimpleInstruction::J { target } |
				SimpleInstruction::Jal { target } |
				SimpleInstruction::Beq { target, .. } |
				SimpleInstruction::Bne { target, .. } |
				SimpleInstruction::Bltz { target, .. } |
				SimpleInstruction::Bgez { target, .. } |
				SimpleInstruction::Bgtz { target, .. } |
				SimpleInstruction::Blez { target, .. } |
				SimpleInstruction::Bltzal { target, .. } |
				SimpleInstruction::Bgezal { target, .. },
			) => {
				if let Some(func) = functions.get(*target) {
					print!(" # {}", func.name);
				}
				if let Some(local_idx) = locals_pos.get(target) {
					print!(" # .{local_idx}");
				}
			},

			// Comment returns
			Instruction::Simple(SimpleInstruction::Jr { rs: Register::Ra }) => {
				print!(" # Return");
			},

			// Comment loading address, loading and writing values of string and data
			// TODO: Maybe check loads / writes to halfway between
			//       the strings / data.
			Instruction::Pseudo(
				PseudoInstruction::La { target: offset, .. } |
				PseudoInstruction::Li32 { imm: offset, .. } |
				PseudoInstruction::LbImm { offset, .. } |
				PseudoInstruction::LbuImm { offset, .. } |
				PseudoInstruction::LhImm { offset, .. } |
				PseudoInstruction::LhuImm { offset, .. } |
				PseudoInstruction::LwlImm { offset, .. } |
				PseudoInstruction::LwImm { offset, .. } |
				PseudoInstruction::LwrImm { offset, .. } |
				PseudoInstruction::SbImm { offset, .. } |
				PseudoInstruction::ShImm { offset, .. } |
				PseudoInstruction::SwlImm { offset, .. } |
				PseudoInstruction::SwImm { offset, .. } |
				PseudoInstruction::SwrImm { offset, .. },
			) => {
				if let Some(string_idx) = strings_pos.get(Pos::ref_cast(offset)) {
					print!(" # string_{string_idx}");
				}
				if let Some(data_idx) = data_pos.get(Pos::ref_cast(offset)) {
					print!(" # data_{data_idx}");
				}
			},

			// Comment `dw`s with both function and data
			Instruction::Directive(Directive::Dw(offset) | Directive::DwRepeated { value: offset, .. }) => {
				print!(" #");
				if let Some(func) = functions.get(Pos(*offset)) {
					print!(" # {}", func.name);
				}
				if let Some(local_idx) = locals_pos.get(Pos::ref_cast(offset)) {
					print!(" # .{local_idx}");
				}
				if let Some(string_idx) = strings_pos.get(Pos::ref_cast(offset)) {
					print!(" # string_{string_idx}");
				}
				if let Some(data_idx) = data_pos.get(Pos::ref_cast(offset)) {
					print!(" # data_{data_idx}");
				}
			},

			_ => (),
		}

		// Append any comments in this line
		if let Some(cur_func) = cur_func {
			if let Some(comment) = cur_func.comments.get(&cur_pos) {
				print!(" # {comment}");
			}
		}
		// And finish the line
		println!();

		// If the last instruction was a `return` and we have a function, space it out
		if let (Some(Instruction::Simple(SimpleInstruction::Jr { rs: Register::Ra })), Some(_cur_func)) = (last_instruction, cur_func) {
			println!();
			println!("####################");
		}
	}

	Ok(())
}
