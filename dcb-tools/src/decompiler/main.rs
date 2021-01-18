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
// We're fine with it
#![allow(clippy::match_bool)]

// Modules
mod cli;
#[path = "../logger.rs"]
mod logger;

// Imports
use std::fmt;

use anyhow::Context;
use dcb_exe::{
	exe::{
		inst::{basic, pseudo, Directive, Inst, InstFmt, InstTarget, InstTargetFmt},
		iter::ExeItem,
		Func,
	},
	Exe, Pos,
};
use dcb_io::GameFile;
use dcb_iso9660::CdRom;

fn main() -> Result<(), anyhow::Error> {
	// Initialize our logger.
	logger::init();

	// Get all data from cli
	let cli = cli::CliData::new();

	// Open the game file
	let input_file = std::fs::File::open(&cli.game_file_path).context("Unable to open input file")?;
	let mut cdrom = CdRom::new(input_file);
	let mut game_file = GameFile::new(&mut cdrom).context("Unable to read game file")?;

	// Read the executable
	log::debug!("Deserializing executable");
	let exe = Exe::deserialize(&mut game_file).context("Unable to parse game executable")?;

	if cli.print_header {
		println!("Header:\n{}", exe.header());
	}

	for item in exe.iter() {
		match item {
			// For each function or header, print a header and all it's instructions
			ExeItem::Func { func, insts } => {
				println!("\n##########");
				println!("{}:", func.name);
				if !func.signature.is_empty() {
					println!("# {}", func.signature);
				}
				for description in func.desc.lines() {
					println!("# {description}");
				}
				for (pos, inst) in insts {
					// If there's a label, print it
					if let Some(label) = func.labels.get(&pos) {
						println!("\t.{label}:");
					}

					// Write the position
					if cli.print_inst_pos {
						print!("{pos}:");
					}

					// Write the instruction
					print!("\t{}", self::inst_display(&inst, &exe, Some(func), pos));

					// If there's a comment, print it
					if let Some(comment) = func.comments.get(&pos) {
						print!(" # {comment}");
					}

					println!();
				}
				println!("##########\n");
			},

			ExeItem::Data { data, insts } => {
				println!("\n##########");
				println!("{}:", data.name());
				for description in data.desc().lines() {
					println!("# {description}");
				}
				for (pos, inst) in insts {
					// Write the position
					if cli.print_inst_pos {
						print!("{pos}:");
					}

					// Write the instruction
					print!("\t{}", self::inst_display(&inst, &exe, None, pos));

					println!();
				}
				println!("##########\n");
			},

			// If it's standalone, print it by it's own
			ExeItem::Unknown { insts } => {
				for (pos, inst) in insts {
					// Write the position
					if cli.print_inst_pos {
						print!("{pos}: ");
					}

					// Write the instruction
					print!("{}", self::inst_display(&inst, &exe, None, pos));

					println!();
				}
			},
		}
	}

	if cli.print_data_table {
		println!("Data Table:\n{}", exe.data_table());
	}

	Ok(())
}

/// Returns a display-able for an instruction inside a possible function
#[must_use]
pub fn inst_display<'a>(inst: &'a Inst, exe: &'a Exe, func: Option<&'a Func>, pos: Pos) -> impl fmt::Display + 'a {
	// Overload the target of as many as possible using `inst_target`.
	dcb_util::DisplayWrapper::new(move |f| match inst {
		Inst::Basic(basic::Inst::Cond(inst)) => write!(f, "{}", self::inst_target_fmt(inst, pos, self::inst_target(exe, func, inst.target(pos)))),
		Inst::Basic(basic::Inst::Jmp(basic::jmp::Inst::Imm(inst))) => {
			write!(f, "{}", self::inst_target_fmt(inst, pos, self::inst_target(exe, func, inst.target(pos))))
		},
		Inst::Pseudo(pseudo::Inst::LoadImm(
			inst
			@ pseudo::load_imm::Inst {
				kind: pseudo::load_imm::Kind::Address(Pos(target)) | pseudo::load_imm::Kind::Word(target),
				..
			},
		)) => write!(f, "{}", self::inst_target_fmt(inst, pos, self::inst_target(exe, func, Pos(*target)))),
		Inst::Pseudo(pseudo::Inst::Load(inst)) => write!(f, "{}", self::inst_target_fmt(inst, pos, self::inst_target(exe, func, inst.target(pos)))),
		Inst::Pseudo(pseudo::Inst::Store(inst)) => write!(f, "{}", self::inst_target_fmt(inst, pos, self::inst_target(exe, func, inst.target(pos)))),
		Inst::Directive(directive @ Directive::Dw(target)) => {
			write!(f, "{}", self::inst_target_fmt(directive, pos, self::inst_target(exe, func, Pos(*target))))
		},
		// TODO: Directive
		inst => write!(f, "{}", self::inst_fmt(inst, pos)),
	})
}

/// Looks up a function, data or label, if possible, else returns the position.
#[must_use]
pub fn inst_target<'a>(exe: &'a Exe, func: Option<&'a Func>, pos: Pos) -> impl fmt::Display + 'a {
	dcb_util::DisplayWrapper::new(move |f| {
		// Try to get a label for the current function, if it exists
		if let Some(label) = func.and_then(|func| func.labels.get(&pos)) {
			return write!(f, ".{}", label);
		}

		// Try to get a function from it
		if let Some(func) = exe.func_table().get_containing(pos) {
			// And then one of it's labels
			if let Some(label) = func.labels.get(&pos) {
				return write!(f, "{}.{}", func.name, label);
			}

			// Or just any position in it
			return match func.start_pos == pos {
				true => write!(f, "{}", func.name),
				false => write!(f, "{}{:+#x}", func.name, pos - func.start_pos),
			};
		}

		// Else try a data
		if let Some(data) = exe.data_table().get_containing(pos) {
			return match data.start_pos() == pos {
				true => write!(f, "{}", data.name()),
				false => write!(f, "{}{:+#x}", data.name(), pos - data.start_pos()),
			};
		}

		// Or just return the position itself
		write!(f, "{}", pos)
	})
}

/// Helper function to display an instruction using `InstFmt`
#[must_use]
pub fn inst_fmt(inst: &impl InstFmt, pos: Pos) -> impl fmt::Display + '_ {
	dcb_util::DisplayWrapper::new(move |f| inst.fmt(pos, f))
}

/// Helper function to display an instruction using `InstTargetFmt`
#[must_use]
pub fn inst_target_fmt<'a>(inst: &'a impl InstTargetFmt, pos: Pos, target: impl fmt::Display + 'a) -> impl fmt::Display + 'a {
	dcb_util::DisplayWrapper::new(move |f| inst.fmt(pos, &target, f))
}
