//! Decompiler

#![feature(
	box_syntax,
	backtrace,
	panic_info_message,
	array_chunks,
	format_args_capture,
	bindings_after_at,
	iter_map_while
)]

// Modules
mod cli;

// Imports
use anyhow::Context;
use dcb_exe::{
	inst::{basic, pseudo, Directive, Inst, InstFmt, InstTarget, InstTargetFmt},
	reader::iter::ExeItem,
	ExeReader, Func, Pos,
};
use std::{collections::BTreeMap, fmt, path::PathBuf};

fn main() -> Result<(), anyhow::Error> {
	// Initialize the logger
	simplelog::TermLogger::init(log::LevelFilter::Info, simplelog::Config::default(), simplelog::TerminalMode::Stderr)
		.expect("Unable to initialize logger");

	// Get all data from cli
	let cli = cli::CliData::new();

	// Open the input file
	let mut input_file = std::fs::File::open(&cli.input_path).context("Unable to open input file")?;

	// Read the executable
	log::debug!("Deserializing executable");
	let exe = ExeReader::deserialize(&mut input_file).context("Unable to parse game executable")?;

	if cli.print_header {
		let header_file_path = {
			let mut path = cli.input_path.clone().into_os_string();
			path.push(".header");
			PathBuf::from(path)
		};
		let header_file = std::fs::File::create(header_file_path).context("Unable to create header file")?;
		serde_yaml::to_writer(header_file, exe.header()).context("Unable to write header to file")?;
	}

	// Instruction buffers
	let mut inst_buffers: BTreeMap<Pos, String> = BTreeMap::new();

	// If currently in an inline-comment alignment
	let mut cur_inline_comment_alignment_max_inst_len: Option<usize> = None;

	for item in exe.iter() {
		match item {
			// For each function or header, print a header and all it's instructions
			ExeItem::Func { func, insts } => {
				// Drop any old instruction buffers
				inst_buffers = inst_buffers.split_off(&func.start_pos);

				println!("\n##########");
				println!("{}:", func.name);
				if !func.signature.is_empty() {
					println!("# {}", func.signature);
				}
				for description in func.desc.lines() {
					println!("# {description}");
				}

				let insts: Vec<_> = insts.collect();
				for (cur_n, (pos, inst)) in insts.iter().enumerate() {
					// If there's a comment, print it
					if let Some(comment) = func.comments.get(pos) {
						// Iterate over the lines in the comment
						for line in comment.lines() {
							println!("# {line}");
						}
					}

					// If there's a label, print it
					if let Some(label) = func.labels.get(pos) {
						println!("\t.{label}:");
					}

					// If we don't have a comment, remove the current alignment
					if !func.inline_comments.contains_key(&pos) {
						cur_inline_comment_alignment_max_inst_len = None;
					}

					// If we don't have any alignment padding, and this instruction and the next have inline comments,
					// set the inline alignment
					if cur_inline_comment_alignment_max_inst_len.is_none() &&
						func.inline_comments.contains_key(&pos) &&
						func.inline_comments.contains_key(&(pos + 4))
					{
						let max_inst_len = (0..)
							.map_while(|n| {
								// If the next instruction doesn't have a comment, return
								let offset = 4 * n;
								let pos = pos + offset;
								if !func.inline_comments.contains_key(&pos) {
									return None;
								}

								// Then build the instruction
								let inst = &insts.get(cur_n + n)?.1;
								let inst = inst_buffers
									.entry(pos)
									.or_insert_with(|| self::inst_display(inst, &exe, Some(func), pos).to_string());
								let inst_len = inst.len();

								Some(inst_len)
							})
							.max()
							.expect("Next instruction had an inline comment");

						cur_inline_comment_alignment_max_inst_len = Some(max_inst_len);
					}

					// Write the position
					if cli.print_inst_pos {
						print!("{pos}:");
					}

					// If we have the instruction buffer, pop it and use it
					match inst_buffers.get(&pos) {
						Some(inst) => print!("\t{inst}"),
						None => print!("\t{}", self::inst_display(&inst, &exe, Some(func), *pos)),
					}

					// If there's an inline comment, print it
					if let Some(comment) = func.inline_comments.get(&pos) {
						// Replace any newlines with '\n'
						let modified_comment;
						let comment = match comment.contains('\n') {
							true => {
								modified_comment = comment.replace("\n", "\\n");
								&modified_comment
							},
							false => comment,
						};

						// If we have alignment padding, apply it
						if let Some(max_inst_len) = cur_inline_comment_alignment_max_inst_len {
							let inst = inst_buffers
								.get(&pos)
								.expect("Instruction wasn't in buffer during inline comment alignment");
							let padding = max_inst_len - inst.len();
							for _ in 0..padding {
								print!(" ");
							}
						}

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
pub fn inst_display<'a>(inst: &'a Inst, exe: &'a ExeReader, func: Option<&'a Func>, pos: Pos) -> impl fmt::Display + 'a {
	// Overload the target of as many as possible using `inst_target`.
	dcb_util::DisplayWrapper::new(move |f| match inst {
		Inst::Basic(basic::Inst::Cond(inst)) => write!(f, "{}", self::inst_target_fmt(inst, pos, self::inst_target(exe, func, inst.target(pos)))),
		Inst::Basic(basic::Inst::Jmp(basic::jmp::Inst::Imm(inst))) => {
			write!(f, "{}", self::inst_target_fmt(inst, pos, self::inst_target(exe, func, inst.target(pos))))
		},
		Inst::Pseudo(pseudo::Inst::LoadImm(
			inst @ pseudo::load_imm::Inst {
				kind: pseudo::load_imm::Kind::Address(Pos(target)),
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
pub fn inst_target<'a>(exe: &'a ExeReader, func: Option<&'a Func>, pos: Pos) -> impl fmt::Display + 'a {
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
