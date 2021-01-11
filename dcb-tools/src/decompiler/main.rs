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
use anyhow::Context;
use dcb_exe::exe::inst::InstFmt;
use dcb_io::GameFile;

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
	let exe = dcb_exe::Exe::deserialize(&mut game_file).context("Unable to parse game executable")?;

	println!("Header:\n{}", exe.header());

	for (pos, inst) in exe.parse_iter() {
		println!("{}: {}", pos, inst.fmt_value(pos));
	}

	/*
	for item in exe.iter() {
		match item {
			// For each function or header, print a header and all it's instructions
			ExeItem::Func(func) => {
				println!("####################");
				println!("{}:", func.name);
				if !func.signature.is_empty() {
					println!("# {}", func.signature);
				}
				for description in func.desc.lines() {
					println!("# {}", description);
				}
				/*
				#[allow(
					clippy::as_conversions,
					clippy::cast_precision_loss,
					clippy::cast_possible_truncation,
					clippy::cast_sign_loss
				)] // TODO: Check if this is fine
				let pos_width = ((func.end_pos - func.start_pos) as f64).log10() as usize;
				for pos in (func.start_pos.0..func.end_pos.0).map(Pos) {
					let inst = exe.get(pos).expect("Unable to get function instruction");
					println!(
						"{:0width$}: {}",
						SignedHex(pos - func.start_pos),
						inst.fmt_value(pos, &*exe.bytes),
						width = pos_width,
					);
				}
				*/
				println!("####################");
			},

			ExeItem::Data(data) => {
				println!("####################");
				println!("{}:", data.name);
				for description in data.desc.lines() {
					println!("# {}", description);
				}
				/*
				#[allow(
					clippy::as_conversions,
					clippy::cast_precision_loss,
					clippy::cast_possible_truncation,
					clippy::cast_sign_loss
				)] // TODO: Check if this is fine
				let pos_width = f64::from(data.size()).log10() as usize;
				for pos in (data.pos.0..data.end_pos().0).map(Pos) {
					let inst = exe.get(pos).expect("Unable to get data instruction");
					println!(
						"{:0width$}: {}",
						SignedHex(pos - data.pos),
						inst.fmt_value(pos, &*exe.bytes),
						width = pos_width,
					);
				}
				*/
				println!("####################");
			},

			// If it's standalone, print it by it's own
			ExeItem::Inst(pos, inst) => {
				println!("{}: {}", pos, inst.fmt_value(pos));
			},
		}
	}
	*/

	/*
	// Build the full instructions iterator
	// TODO: Revamp this, iterate over an enum of `Func | Data | Other`
	let full_iter = functions
		.with_instructions(instructions.iter().map(|(pos, instruction)| (*pos, instruction)))
		.scan(None, |last_instruction, output @ (_, cur_instruction, _)| {
			Some((output, last_instruction.replace(cur_instruction)))
		})
		.map(|((cur_pos, instruction, cur_func), last_instruction)| (cur_pos, instruction, last_instruction, cur_func))
		.scan(None, |last_func, output @ (_, _, _, cur_func)| {
			Some((output, match cur_func {
				Some(cur_func) => last_func.replace(cur_func),
				None => *last_func,
			}))
		})
		.map(|((cur_pos, instruction, last_instruction, cur_func), last_func)| (cur_pos, instruction, last_instruction, cur_func, last_func));

	// Read all instructions
	let mut skipped_nops = 0;
	for (cur_pos, instruction, last_instruction, cur_func, last_func) in full_iter {
		// Note: Required by `rust-analyzer` currently, it can't determine the type of `cur_func`.
		let cur_func: Option<&Func> = cur_func;
		let last_func: Option<&Func> = last_func;

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

		// If we just exited a function, space it out.
		if let Some(last_func) = last_func {
			if last_func.end_pos == cur_pos {
				println!("####################");
				println!();
			}
		}

		// Space out data if it had a name
		if let Some(data) = data_pos.get(cur_pos - 1) {
			if data.end_pos() == cur_pos && !data.name.is_empty() {
				println!("####################");
				println!();
			}
		}

		// Check if we need to prefix
		if let Some(cur_func) = cur_func {
			if cur_func.start_pos == cur_pos {
				println!();
				println!("####################");
				println!("{}:", cur_func.name);
				if !cur_func.signature.is_empty() {
					println!("# {}", cur_func.signature);
				}
				for description in cur_func.desc.lines() {
					println!("# {}", description);
				}
			}
			if let Some(label) = cur_func.labels.get(&cur_pos) {
				println!("\t.{label}:");
			}
		}
		if let Some(data) = data_pos.get(cur_pos) {
			if data.pos == cur_pos {
				println!();
				println!("####################");
				println!("{}:", data.name);
				for description in data.desc.lines() {
					println!("# {}", description);
				}
			}
		}

		// Print the instruction and it's location.
		match cur_func {
			Some(cur_func) => print!("{:#05x}:\t", cur_pos - cur_func.start_pos),
			None => print!("{cur_pos:#010x}:\t"),
		}
		match instruction {
			/* Instruction::Basic(
				BasicInst::J { target } |
				BasicInst::Jal { target } |
				BasicInst::Beq { target, .. } |
				BasicInst::Bne { target, .. } |
				BasicInst::Bltz { target, .. } |
				BasicInst::Bgez { target, .. } |
				BasicInst::Bgtz { target, .. } |
				BasicInst::Blez { target, .. } |
				BasicInst::Bltzal { target, .. } |
				BasicInst::Bgezal { target, .. },
			) | */
			Instruction::Pseudo(
				PseudoInstruction::B { target } | PseudoInstruction::Beqz { target, .. } | PseudoInstruction::Bnez { target, .. },
			) => match functions
				.get(*target)
				.map(|func| (&func.name, ""))
				.or_else(|| cur_func.and_then(|func| func.labels.get(target).map(|label| (label, "."))))
			{
				Some((target, prefix)) => print!("{}", set_instruction_immediate(instruction, format_args!(" {prefix}{target}"))),
				None => print!("{instruction}"),
			},

			// Comment loading address, loading and writing values of string and data
			Instruction::Pseudo(
				PseudoInstruction::La { target, .. } |
				PseudoInstruction::Li32 { imm: target, .. } |
				PseudoInstruction::LbImm { offset: target, .. } |
				PseudoInstruction::LbuImm { offset: target, .. } |
				PseudoInstruction::LhImm { offset: target, .. } |
				PseudoInstruction::LhuImm { offset: target, .. } |
				PseudoInstruction::LwlImm { offset: target, .. } |
				PseudoInstruction::LwImm { offset: target, .. } |
				PseudoInstruction::LwrImm { offset: target, .. } |
				PseudoInstruction::SbImm { offset: target, .. } |
				PseudoInstruction::ShImm { offset: target, .. } |
				PseudoInstruction::SwlImm { offset: target, .. } |
				PseudoInstruction::SwImm { offset: target, .. } |
				PseudoInstruction::SwrImm { offset: target, .. },
			) |
			Instruction::Directive(Directive::Dw(target)) => match functions
				.get(Pos(*target))
				.map(|func| (func.start_pos, &func.name))
				.or_else(|| data_pos.get(Pos(*target)).map(|data| (data.pos, &data.name)))
			{
				Some((start_pos, name)) => {
					if start_pos == Pos(*target) {
						print!("{}", set_instruction_immediate(instruction, format_args!(" {name}")));
					} else {
						let offset = Pos(*target) - start_pos;
						if offset > 0 {
							print!("{}", set_instruction_immediate(instruction, format_args!(" {name} + {offset:#x}")));
						}
					}
				},
				None => print!("{instruction}"),
			},

			_ => print!("{instruction}"),
		}

		// Append any comments in this line
		if let Some(cur_func) = cur_func {
			if let Some(comment) = cur_func.comments.get(&cur_pos) {
				print!(" # {comment}");
			}
		}

		// And finish the line
		println!();
	}
	*/

	Ok(())
}

/*
/// Helper function to modify the immediate argument from an instruction
// TODO: Use something better than this
fn set_instruction_immediate(instruction: &Instruction, fmt: std::fmt::Arguments) -> String {
	use std::fmt::Write;

	// Get the string as a string
	let mut s: String = instruction.to_string();

	// If the instruction is of the form `.. , imm(..)`, then grab
	// the `(..)`
	#[allow(clippy::indexing_slicing)] // This can't panic
	let parens = s.rfind('(').map(|idx| s[idx..].to_string());

	// Truncate the string from it's last argument and write the fmt
	// Note: We use ' ' instead of ',' to support single argument instructions.
	s.truncate(s.rfind(' ').unwrap_or(0));
	s.write_fmt(fmt).expect("Unable to write format to string");

	// If we had a paren, write it
	// Note: This technically shouldn't be needed, but just in case.
	if let Some(parens) = parens {
		s.push_str(&parens);
	}

	s
}
*/
