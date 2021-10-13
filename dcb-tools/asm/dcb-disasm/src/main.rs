//! Disassembler

#![feature(try_blocks, format_args_capture, btree_drain_filter)]

// Modules
mod args;
mod display_ctx;
mod external;

// Exports
use display_ctx::DisplayCtx;
use external::ExternalResources;

// Imports
use anyhow::Context;
use dcb_exe::{
	func::ArgPos,
	inst::{parse::LineArgExpr, Inst, InstDisplay, InstFmtArg, ParseCtx},
	reader::{iter::ExeItem, DeserializeOpts},
	ExeReader, Func, Pos,
};
use itertools::{Itertools, Position};
use std::{collections::BTreeMap, fmt, fs};

fn main() -> Result<(), anyhow::Error> {
	// Initialize the logger
	simplelog::TermLogger::init(
		log::LevelFilter::Info,
		simplelog::Config::default(),
		simplelog::TerminalMode::Stderr,
	)
	.expect("Unable to initialize logger");

	// Get all data from cli
	let args = args::Args::get();

	// Load all external resources
	let ExternalResources { data_table, func_table } = ExternalResources::load(&args);

	// Log how much we loaded
	log::info!("Loaded {} data entries", data_table.len());
	log::info!("Loaded {} functions", func_table.len());

	// Open the input file
	let input_file_path = &args.input_path;
	let mut input_file =
		fs::File::open(input_file_path).with_context(|| format!("Unable to open input file {input_file_path:?}"))?;

	// Read the executable
	log::debug!("Deserializing executable");
	let exe = ExeReader::deserialize(&mut input_file, DeserializeOpts {
		data_table: Some(data_table),
		func_table: Some(func_table),
	})
	.context("Unable to parse game executable")?;

	// If we should print a header, create a `.header` file with the input
	if let Some(header_path) = &args.header_path {
		zutil::write_to_file(header_path, exe.header(), serde_yaml::to_writer)
			.with_context(|| format!("Unable to write header to file {header_path:?}"))?;
	}

	// Instruction display buffer for calculating alignment
	// during inline instructions
	let mut inst_display_cache: BTreeMap<Pos, String> = BTreeMap::new();

	// The maximum instruction length for the current inline comment run
	let mut cur_inline_comment_run_max_inst_len: Option<usize> = None;

	// Current function instructions
	let mut cur_func_insts = BTreeMap::new();

	// Current function instruction argument overrides
	let mut cur_func_inst_arg_overrides = BTreeMap::new();

	// Display all items in the executable
	for item in exe.iter() {
		self::display_item(
			item,
			&exe,
			&mut inst_display_cache,
			&mut cur_func_insts,
			&mut cur_func_inst_arg_overrides,
			&mut cur_inline_comment_run_max_inst_len,
			&args,
		);
	}

	Ok(())
}

/// Displays an executable item
fn display_item<'a>(
	item: ExeItem<'a>, exe: &'a ExeReader, inst_display_cache: &mut BTreeMap<Pos, String>,
	cur_func_insts: &mut BTreeMap<Pos, Inst<'a>>, cur_func_inst_arg_overrides: &mut BTreeMap<ArgPos, String>,
	cur_inline_comment_run_max_inst_len: &mut Option<usize>, args: &args::Args,
) {
	match item {
		// For each function or header, print a header and all it's instructions
		ExeItem::Func { func, insts } => {
			// Drop any old instruction buffers
			inst_display_cache.drain_filter(|&pos, _| pos < func.start_pos);

			// Clear the previous function's instruction and append the new ones
			cur_func_insts.clear();
			cur_func_insts.extend(insts);

			// Get the argument overrides
			cur_func_inst_arg_overrides.clone_from(&func.inst_arg_overrides);

			println!("\n##########");
			println!("{}:", func.name);
			if !func.signature.is_empty() {
				println!("# {}", func.signature);
			}
			for description in func.desc.lines() {
				println!("# {description}");
			}

			let insts = &*cur_func_insts;
			let inst_arg_overrides = cur_func_inst_arg_overrides;
			for (pos, inst) in insts {
				// If there's a block comment, print it
				if let Some(comment) = func.block_comments.get(pos) {
					for line in comment.lines() {
						println!("# {line}");
					}
				}

				// If there's a label, print it
				if let Some(label) = func.labels.get(pos) {
					println!("\t.{label}:");
				}

				// If we don't have a comment, remove the current alignment
				if !func.inline_comments.contains_key(pos) {
					*cur_inline_comment_run_max_inst_len = None;
				}

				// If we don't have any alignment padding, and this instruction and the next have inline comments,
				// set the inline alignment
				if cur_inline_comment_run_max_inst_len.is_none() &&
					func.inline_comments.contains_key(pos) &&
					func.inline_comments.contains_key(&(pos + 4))
				{
					let max_inst_len = (0..)
						.map(|n| pos + 4 * n)
						.map_while(|pos| {
							// If the next instruction doesn't have a comment, return
							if !func.inline_comments.contains_key(&pos) {
								return None;
							}

							// Then build the instruction
							let inst = &insts.get(&pos)?;
							let inst = inst_display_cache.entry(pos).or_insert_with(|| {
								self::inst_display(inst, exe, Some(func), Some(inst_arg_overrides), pos).to_string()
							});
							let mut inst_len = inst.len();

							// If we had a branch / jump instruction before this one, add the "+ " length
							if insts.get(&(pos - 4)).map_or(false, |inst| inst.expects_branch_delay()) {
								inst_len += 2;
							}

							Some(inst_len)
						})
						.max()
						.expect("Next instruction had an inline comment");

					*cur_inline_comment_run_max_inst_len = Some(max_inst_len);
				}

				// Write the position
				if args.print_inst_pos {
					print!("{pos}:");
				}

				// Add a tab before any instruction in a function
				print!("\t");

				// If we had a branch / jump instruction before this one, add a "+ "
				let is_branch_delay = insts.get(&(pos - 4)).map_or(false, |inst| inst.expects_branch_delay());
				if is_branch_delay {
					print!("+ ");
				}

				// If we have the instruction buffer, pop it and use it
				match inst_display_cache.get(pos) {
					Some(inst) => print!("{inst}"),
					None => print!(
						"{}",
						self::inst_display(inst, exe, Some(func), Some(inst_arg_overrides), *pos)
					),
				}

				// If there's an inline comment, print it
				if let Some(comment) = func.inline_comments.get(pos) {
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
					if let Some(max_inst_len) = *cur_inline_comment_run_max_inst_len {
						let inst = inst_display_cache
							.get(pos)
							.expect("Instruction wasn't in buffer during inline comment alignment");
						let padding = max_inst_len - inst.len() - if is_branch_delay { 2 } else { 0 };
						for _ in 0..padding {
							print!(" ");
						}
					}

					print!(" # {comment}");
				}

				println!();
			}
			println!("##########\n");

			// If there are any leftover overrides, warn
			inst_arg_overrides.drain_filter(|pos, s| {
				log::warn!("Ignoring override at {}/{}: {}", pos.pos, pos.arg, s.escape_debug());
				true
			});
		},

		ExeItem::Data { data, insts } => {
			println!("\n##########");
			println!("{}:", data.name());
			for description in data.desc().lines() {
				println!("# {description}");
			}
			for (pos, inst) in insts {
				// Write the position
				if args.print_inst_pos {
					print!("{pos}:");
				}

				// Write the instruction
				print!("\t{}", self::inst_display(&inst, exe, None, None, pos));

				println!();
			}
			println!("##########\n");
		},

		// If it's standalone, print it by it's own
		ExeItem::Unknown { insts } => {
			let mut prev_inst = None;
			for (pos, inst) in insts {
				// Write the position
				if args.print_inst_pos {
					print!("{pos}: ");
				}

				// If we had a branch / jump instruction before this one, add a "+ "
				if prev_inst.as_ref().map_or(false, Inst::expects_branch_delay) {
					print!("+ ");
				}

				// Write the instruction
				print!("{}", self::inst_display(&inst, exe, None, None, pos));

				println!();

				prev_inst = Some(inst);
			}
		},
	}
}

/// Returns a display-able for an instruction inside a possible function
#[must_use]
pub fn inst_display<'a>(
	inst: &'a Inst, exe: &'a ExeReader, func: Option<&'a Func>,
	mut inst_arg_overrides: Option<&'a mut BTreeMap<ArgPos, String>>, pos: Pos,
) -> impl fmt::Display + 'a {
	// Overload the target of as many as possible using `inst_target`.
	zutil::DisplayWrapper::new(move |f| {
		// Build the context and get the mnemonic + args
		let ctx = DisplayCtx::new(exe, func, pos);
		let mnemonic = inst.mnemonic(&ctx);
		let args = inst.args(&ctx);

		write!(f, "{mnemonic}")?;
		for (idx, arg) in args.with_position().enumerate() {
			// Write ',' if it's first, then a space
			match &arg {
				Position::First(_) | Position::Only(_) => write!(f, " "),
				_ => write!(f, ", "),
			}?;
			let arg = arg.into_inner();

			// If we have an override for this argument, use it
			match inst_arg_overrides
				.as_mut()
				.and_then(|arg_overrides| arg_overrides.remove(&ArgPos { pos, arg: idx }))
			{
				Some(value) => {
					// Validator
					let validate = || -> Result<(), anyhow::Error> {
						// Parse the override
						let (expr, rest) = LineArgExpr::parse(&value).context("Unable to parse override")?;

						let rest = rest.trim_start();
						anyhow::ensure!(rest.is_empty(), "Leftover tokens after parsing override: {:?}", rest);

						// Then evaluate it
						let parse_ctx = OverrideParseCtx { pos, exe };
						let expected_value = parse_ctx.eval_expr(&expr).context("Unable to evaluate override")?;

						// And make sure it's the same as the original
						let actual_value = match arg {
							InstFmtArg::Register(_) => anyhow::bail!("Cannot override register argument"),
							InstFmtArg::RegisterOffset { offset, .. } => offset,
							InstFmtArg::Literal(value) => value,
							InstFmtArg::Target(target) => i64::from(target.0),
							InstFmtArg::String(_) => anyhow::bail!("Cannot override string argument"),
							InstFmtArg::RegArray(_) => anyhow::bail!("Cannot override array argument"),
						};
						anyhow::ensure!(
							actual_value == expected_value,
							"Original value is {}, override is {}",
							actual_value,
							expected_value
						);

						Ok(())
					};

					if let Err(err) = validate() {
						log::warn!("Override for {}/{} failed validation: {:?}", pos, idx, err);
					}

					match arg {
						InstFmtArg::RegisterOffset { register, .. } => write!(f, "{value}({register})")?,
						InstFmtArg::Literal(_) | InstFmtArg::Target(_) => write!(f, "{value}")?,
						_ => arg.write(f, &ctx)?,
					}
				},
				// Else just write it
				None => arg.write(f, &ctx)?,
			}
		}

		Ok(())
	})
}

/// Parsing context for overrides
pub struct OverrideParseCtx<'a> {
	/// Position
	pos: Pos,

	/// Executable
	exe: &'a ExeReader,
}

impl<'a> ParseCtx<'a> for OverrideParseCtx<'a> {
	fn cur_pos(&self) -> Pos {
		self.pos
	}

	fn label_pos(&self, label: &str) -> Option<Pos> {
		// If a function has the same name, or one of it's labels matches, return it
		if let Some(pos) = self.exe.func_table().range(..).find_map(|func| {
			let warn_on_heuristic = || {
				// If we're asked for a heuristically found function, warn
				if func.kind.is_heuristics() {
					log::warn!(
						"Override parsing context was queried for a heuristically found func: {} @ {}",
						func.name,
						func.start_pos,
					);
				}
			};

			if func.name == label {
				warn_on_heuristic();
				return Some(func.start_pos);
			}

			match label.split_once('.') {
				Some((func_name, func_label)) if func_name == func.name => {
					func.labels.range(..).find_map(|(&pos, label_name)| {
						(label_name == func_label).then(|| {
							warn_on_heuristic();
							pos
						})
					})
				},
				_ => None,
			}
		}) {
			return Some(pos);
		}

		// If a data has the same name, return it
		if let Some(data) = self.exe.data_table().search_name(label) {
			// If we're asked for a heuristically found data, warn
			if data.kind().is_heuristics() {
				log::warn!(
					"Override parsing context was queried for a heuristically found data: {}",
					data
				);
			}

			return Some(data.start_pos());
		}

		None
	}
}
