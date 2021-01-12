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
		inst::{basic, pseudo, Inst, InstFmt, InstTarget, InstTargetFmt},
		iter::ExeItem,
		Func,
	},
	Exe, Pos,
};
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
	let exe = Exe::deserialize(&mut game_file).context("Unable to parse game executable")?;

	println!("Header:\n{}", exe.header());

	for item in exe.iter() {
		match item {
			// For each function or header, print a header and all it's instructions
			ExeItem::Func { func, insts } => {
				println!();
				println!("{}:", func.name);
				if !func.signature.is_empty() {
					println!("# {}", func.signature);
				}
				for description in func.desc.lines() {
					println!("# {description}");
				}
				for (pos, label) in &func.labels {
					println!("# {pos}: .{label}");
				}
				for (pos, inst) in insts {
					// If there's a label, print it
					if let Some(label) = func.labels.get(&pos) {
						println!("\t.{label}:");
					}

					// Write the position
					print!("{pos}: {}", self::inst_display(&inst, &exe, Some(func), pos));

					// If there's a comment, print it
					if let Some(comment) = func.comments.get(&pos) {
						print!(" # {comment}");
					}

					println!();
				}
			},

			ExeItem::Data { data, insts } => {
				println!();
				println!("{}:", data.name);
				for description in data.desc.lines() {
					println!("# {description}");
				}
				for (pos, inst) in insts {
					println!("{}: {}", pos, self::inst_display(&inst, &exe, None, pos));
				}
			},

			// If it's standalone, print it by it's own
			ExeItem::Unknown { insts } => {
				for (pos, inst) in insts {
					println!("{pos}: {}", self::inst_display(&inst, &exe, None, pos));
				}
			},
		}
	}

	Ok(())
}

/// Returns a display-able for an instruction inside a possible function
#[must_use]
#[rustfmt::skip]
pub fn inst_display<'a>(inst: &'a Inst, exe: &'a Exe, func: Option<&'a Func>, pos: Pos) -> impl fmt::Display + 'a {
	dcb_util::DisplayWrapper::new(move |f| {
		match inst {
			Inst::Basic (basic ::Inst::Cond   (inst)) => write!(f, "{}", self::inst_target_fmt(inst, pos, self::inst_target(exe, func, inst.target(pos)))),
			Inst::Basic (basic ::Inst::Jmp    (inst)) => write!(f, "{}", self::inst_fmt(inst, pos)),
			Inst::Basic (basic ::Inst::Load   (inst)) => write!(f, "{}", self::inst_fmt(inst, pos)),
			Inst::Basic (basic ::Inst::Store  (inst)) => write!(f, "{}", self::inst_fmt(inst, pos)),
			Inst::Pseudo(pseudo::Inst::LoadImm(inst)) => write!(f, "{}", self::inst_fmt(inst, pos)),
			Inst::Pseudo(pseudo::Inst::Load   (inst)) => write!(f, "{}", self::inst_fmt(inst, pos)),
			Inst::Pseudo(pseudo::Inst::Store  (inst)) => write!(f, "{}", self::inst_fmt(inst, pos)),
			inst => write!(f, "{}", self::inst_fmt(inst, pos)),
		}
	})
}

/// Looks up a function, data or label, if possible, else returns the position.
#[must_use]
pub fn inst_target<'a>(exe: &'a Exe, func: Option<&'a Func>, pos: Pos) -> impl fmt::Display + 'a {
	dcb_util::DisplayWrapper::new(move |f| {
		if let Some(label) = func.and_then(|func| func.labels.get(&pos)) {
			return write!(f, ".{}", label);
		}

		if let Some(func) = exe.func_table().get(pos) {
			return match func.start_pos == pos {
				true => write!(f, "{}", func.name),
				false => write!(f, "{}{:+#x}", func.name, pos - func.start_pos),
			};
		}

		if let Some(data) = exe.data_table().get(pos) {
			return match data.pos == pos {
				true => write!(f, "{}", data.name),
				false => write!(f, "{}{:+#x}", data.name, pos - data.pos),
			};
		}

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
