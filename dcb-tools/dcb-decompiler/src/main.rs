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
	inst::{DisplayCtx, Inst, InstDisplay},
	reader::iter::ExeItem,
	ExeReader, Func, Pos,
};
use itertools::{Itertools, Position};
use std::{collections::BTreeMap, fmt, path::PathBuf};

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

	// Read all arg overrides
	let inst_arg_overrides_file = std::fs::File::open("resources/inst_args_override.yaml")
		.context("Unable to open instruction args override file")?;
	let inst_arg_overrides: BTreeMap<ArgPos, String> =
		serde_yaml::from_reader(inst_arg_overrides_file).context("Unable to parse instruction args override file")?;

	// Instruction buffer
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
								let inst = inst_buffers.entry(pos).or_insert_with(|| {
									self::inst_display(inst, &exe, Some(func), &inst_arg_overrides, pos).to_string()
								});
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
						None => print!(
							"\t{}",
							self::inst_display(&inst, &exe, Some(func), &inst_arg_overrides, *pos)
						),
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
					print!("\t{}", self::inst_display(&inst, &exe, None, &inst_arg_overrides, pos));

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
					print!("{}", self::inst_display(&inst, &exe, None, &inst_arg_overrides, pos));

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
pub fn inst_display<'a>(
	inst: &'a Inst, exe: &'a ExeReader, func: Option<&'a Func>, inst_arg_overrides: &'a BTreeMap<ArgPos, String>,
	pos: Pos,
) -> impl fmt::Display + 'a {
	// Overload the target of as many as possible using `inst_target`.
	dcb_util::DisplayWrapper::new(move |f| {
		// Build the context and get the mnemonic + args
		let ctx = Ctx { exe, func, pos };
		let mnemonic = inst.mnemonic(&ctx);
		let args = inst.args(&ctx);

		write!(f, "{mnemonic}")?;
		for (idx, arg) in args.with_position().enumerate() {
			// Write ',' if it's first, then a space
			match &arg {
				Position::First(_) | Position::Only(_) => write!(f, " "),
				_ => write!(f, ", "),
			}?;

			// If we have an override for this argument, use it
			match inst_arg_overrides.get(&ArgPos { pos, arg: idx }) {
				Some(arg) => write!(f, "{arg}")?,
				// Else just write it
				None => arg.into_inner().write(f, &ctx)?,
			}
		}

		Ok(())
	})
}

/// Display context
struct Ctx<'a> {
	/// Exe
	exe: &'a ExeReader,

	/// Function
	func: Option<&'a Func>,

	/// Current Position
	pos: Pos,
}

/// Label display for `DisplayCtx::pos_label`
enum LabelDisplay<'a> {
	CurFuncLabel(&'a str),

	OtherFuncLabel { func: &'a str, label: &'a str },

	OtherFunc { func: &'a str },

	Data { name: &'a str },
}

impl<'a> fmt::Display for LabelDisplay<'a> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			LabelDisplay::CurFuncLabel(label) => write!(f, ".{label}"),
			LabelDisplay::OtherFuncLabel { func, label } => write!(f, "{func}.{label}"),
			LabelDisplay::OtherFunc { func } => write!(f, "{func}"),
			LabelDisplay::Data { name } => write!(f, "{name}"),
		}
	}
}

impl<'a> DisplayCtx for Ctx<'a> {
	type Label = LabelDisplay<'a>;

	fn cur_pos(&self) -> Pos {
		self.pos
	}

	fn pos_label(&self, pos: Pos) -> Option<(Self::Label, i64)> {
		// Try to get a label for the current function, if it exists
		if let Some(label) = self.func.and_then(|func| func.labels.get(&pos)) {
			return Some((LabelDisplay::CurFuncLabel(label), 0));
		}

		// Try to get a function from it
		if let Some(func) = self.exe.func_table().get_containing(pos) {
			// And then one of it's labels
			if let Some(label) = func.labels.get(&pos) {
				return Some((
					LabelDisplay::OtherFuncLabel {
						func: &func.name,
						label,
					},
					0,
				));
			}

			// Else just any position in it
			return Some((LabelDisplay::OtherFunc { func: &func.name }, pos - func.start_pos));
		}

		// Else try a data
		if let Some(data) = self.exe.data_table().get_containing(pos) {
			return Some((LabelDisplay::Data { name: data.name() }, pos - data.start_pos()));
		}

		None
	}
}

/// Argument position
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct ArgPos {
	/// Position
	pos: Pos,

	/// Argument
	arg: usize,
}
