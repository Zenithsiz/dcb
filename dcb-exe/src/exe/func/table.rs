//! Function table
//!
//! This module defines the [`FuncTable`] type, which
//! stores all function within the executable.
//!
//! Typically these functions will be a mix of the known function,
//! available through [`FuncTable::known`] and heuristically
//! discovered functions through instruction references, available
//! through [`FuncTable::search_instructions`].

// Modules
pub mod error;
pub mod iter;

// Exports
pub use error::GetKnownError;
pub use iter::WithInstructionsIter;

// Imports
use super::Func;
use crate::{
	exe::{Instruction, Pos},
	util::discarding_sorted_merge_iter::DiscardingSortedMergeIter,
};
use std::{collections::BTreeSet, fs::File, iter::FromIterator};

/// Function table
///
/// Stores all functions sorted by their address.
/// Also guarantees all functions are unique and non-overlapping.
#[derive(PartialEq, Eq, Clone, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct FuncTable(BTreeSet<Func>);

impl FromIterator<Func> for FuncTable {
	fn from_iter<T: IntoIterator<Item = Func>>(iter: T) -> Self {
		Self(iter.into_iter().collect())
	}
}

impl FuncTable {
	/// Merges two data tables, discarding duplicates from `other`.
	///
	/// This can be useful when combining known functions and heuristically
	/// discovered function, as the known functions are always kept, and the
	/// duplicate discovered ones are discarded.
	#[must_use]
	pub fn merge(self, other: Self) -> Self {
		// Note: We don't return the iterator, as we want the user to
		//       keep the guarantees supplied by this type.
		DiscardingSortedMergeIter::new(self.0.into_iter(), other.0.into_iter()).collect()
	}

	/// Retrieves a function with start address `pos`
	#[must_use]
	pub fn get(&self, pos: Pos) -> Option<&Func> {
		// Note: As we're sorted, we can binary search
		self.0.range(..=pos).filter(|func| func.start_pos == pos).next_back()
	}

	/// Adapts an instruction iterator to extract the current function
	pub fn with_instructions<'a, I: Iterator<Item = (Pos, &'a Instruction)>>(&'a self, instructions: I) -> WithInstructionsIter<'a, I> {
		WithInstructionsIter::new(instructions, self)
	}
}

impl FuncTable {
	/// Returns all known functions
	pub fn get_known() -> Result<Self, GetKnownError> {
		let file = File::open("resources/known_funcs.yaml").map_err(GetKnownError::File)?;

		serde_yaml::from_reader(file).map_err(GetKnownError::Parse)
	}

	/// Creates a new list of functions from an iterator over instructions
	#[must_use]
	#[allow(clippy::too_many_lines)] // TODO: Refactor?
	#[allow(clippy::enum_glob_use)] // It's only for this function
	pub fn from_instructions<'a>(_instructions: &(impl Iterator<Item = (Pos, &'a Instruction)> + Clone)) -> Self {
		/*
		// Get all returns
		let returns: BTreeSet<Pos> = instructions
			.clone()
			.filter_map(|(pos, instruction)| match instruction {
				Basic(Jr { rs: Register::Ra }) => Some(pos),
				_ => None,
			})
			.collect();

		// Get all possible tailcalls
		let tailcalls: BTreeSet<Pos> = instructions
			.clone()
			.filter_map(|(pos, instruction)| match instruction {
				Basic(J { .. } | Jr { .. }) => Some(pos),
				_ => None,
			})
			.collect();

		// Get all labels
		let labels: BTreeSet<Pos> = instructions
			.clone()
			.filter_map(|(_, instruction)| match instruction {
				Basic(
					J { target } |
					Beq { target, .. } |
					Bne { target, .. } |
					Bltz { target, .. } |
					Bgez { target, .. } |
					Bgtz { target, .. } |
					Blez { target, .. } |
					Bltzal { target, .. } |
					Bgezal { target, .. },
				) |
				Pseudo(Beqz { target, .. } | Bnez { target, .. } | B { target }) => Some(*target),
				_ => None,
			})
			.filter(|target| (Instruction::CODE_START..Instruction::CODE_END).contains(target))
			.collect();

		// Now check every `Jal` and `Dw` for possible function entrances
		let function_entries: BTreeSet<Pos> = instructions
			.clone()
			.filter_map(|(_, instruction)| match instruction {
				Basic(Jal { target }) => Some(*target),
				Instruction::Directive(Directive::Dw(target)) => Some(Pos(*target)),
				_ => None,
			})
			.filter(|target| (Instruction::CODE_START..Instruction::CODE_END).contains(target))
			.collect();

		#[allow(clippy::cognitive_complexity)] // TODO: Fix
		function_entries
			.iter()
			.zip(0..)
			.map(|(&func_pos, idx)| {
				// Try to get the end position from the returns
				// Note: +8 for return + instruction after.
				let mut end_pos: Pos = returns.range(func_pos..).next().copied().unwrap_or(func_pos) + 8;

				// If there's a function in between us and the return, use the last tailcall instead
				if let Some(next_func_pos) = function_entries.range(func_pos + 4..end_pos).next() {
					end_pos = tailcalls.range(..next_func_pos).next_back().copied().unwrap_or(func_pos) + 8;

					// If we got a tailcall before this function, just end it 2 instructions
					if end_pos <= func_pos {
						end_pos = func_pos + 8;
					}
				}

				// Get all labels within this function
				let labels = labels
					.range(func_pos..end_pos)
					.zip(0..)
					.map(|(&pos, idx)| (pos, format!("{idx}")))
					.collect();

				// Check if any instructions use `$aX` and for what to try and piece
				// together arguments.
				// Arguments `$a0` through `$a3`
				// TODO: Maybe save the instruction iterator for this function in `function_entries` somehow?
				// TODO: Maybe check for return values too.
				let mut arguments: [Option<&'static str>; 4] = [None; 4];
				#[allow(clippy::indexing_slicing)] // The returned indexes will always be < 4.
				for (_, instruction) in instructions
					.clone()
					.skip_while(|(pos, _)| *pos < func_pos)
					.take_while(|(pos, _)| *pos < end_pos)
				{
					// TODO: Generalize this in `Instruction` as a method that
					//       returns all registers used maybe.
					match instruction {
						Basic(Sb { rt, rs, .. } | Lb { rt, rs, .. } | Lbu { rt, rs, .. }) => {
							if let Some(idx) = rt.arg_idx() {
								if arguments[idx].is_none() {
									arguments[idx] = Some("u8");
								}
							}
							if let Some(idx) = rs.arg_idx() {
								if arguments[idx].is_none() {
									arguments[idx] = Some("*u8");
								}
							}
						},
						Pseudo(SbImm { rx, .. } | LbImm { rx, .. } | LbuImm { rx, .. }) => {
							if let Some(idx) = rx.arg_idx() {
								if arguments[idx].is_none() {
									arguments[idx] = Some("*u8");
								}
							}
						},

						Basic(Sh { rt, rs, .. } | Lh { rt, rs, .. } | Lhu { rt, rs, .. }) => {
							if let Some(idx) = rt.arg_idx() {
								if arguments[idx].is_none() {
									arguments[idx] = Some("u16");
								}
							}
							if let Some(idx) = rs.arg_idx() {
								if arguments[idx].is_none() {
									arguments[idx] = Some("*u16");
								}
							}
						},
						Pseudo(ShImm { rx, .. } | LhImm { rx, .. } | LhuImm { rx, .. }) => {
							if let Some(idx) = rx.arg_idx() {
								if arguments[idx].is_none() {
									arguments[idx] = Some("*u16");
								}
							}
						},

						Basic(
							Swl { rt, rs, .. } | Sw { rt, rs, .. } | Swr { rt, rs, .. } | Lwl { rt, rs, .. } | Lw { rt, rs, .. } | Lwr { rt, rs, .. },
						) => {
							if let Some(idx) = rt.arg_idx() {
								if arguments[idx].is_none() {
									arguments[idx] = Some("u32");
								}
							}
							if let Some(idx) = rs.arg_idx() {
								if arguments[idx].is_none() {
									arguments[idx] = Some("*u32");
								}
							}
						},

						Pseudo(
							LwlImm { rx, .. } | LwImm { rx, .. } | LwrImm { rx, .. } | SwlImm { rx, .. } | SwImm { rx, .. } | SwrImm { rx, .. },
						) => {
							if let Some(idx) = rx.arg_idx() {
								if arguments[idx].is_none() {
									arguments[idx] = Some("*u32");
								}
							}
						},

						Basic(
							Addi { rt, rs, .. } |
							Addiu { rt, rs, .. } |
							Slti { rt, rs, .. } |
							Sltiu { rt, rs, .. } |
							Andi { rt, rs, .. } |
							Ori { rt, rs, .. } |
							Xori { rt, rs, .. } |
							Mult { rs, rt } |
							Multu { rs, rt } |
							Div { rs, rt } |
							Divu { rs, rt } |
							Beq { rs, rt, .. } |
							Bne { rs, rt, .. } |
							LwcN { rs, rt, .. } |
							SwcN { rs, rt, .. },
						) |
						Pseudo(Subi { rt, rs, .. } | Subiu { rt, rs, .. }) => {
							if let Some(idx) = rt.arg_idx() {
								if arguments[idx].is_none() {
									arguments[idx] = Some("u32");
								}
							}
							if let Some(idx) = rs.arg_idx() {
								if arguments[idx].is_none() {
									arguments[idx] = Some("u32");
								}
							}
						},

						Basic(
							Add { rd, rs, rt } |
							Addu { rd, rs, rt } |
							Sub { rd, rs, rt } |
							Subu { rd, rs, rt } |
							Slt { rd, rs, rt } |
							Sltu { rd, rs, rt } |
							And { rd, rs, rt } |
							Or { rd, rs, rt } |
							Xor { rd, rs, rt } |
							Nor { rd, rs, rt } |
							Sllv { rd, rt, rs } |
							Srlv { rd, rt, rs } |
							Srav { rd, rt, rs },
						) => {
							if let Some(idx) = rd.arg_idx() {
								if arguments[idx].is_none() {
									arguments[idx] = Some("u32");
								}
							}
							if let Some(idx) = rs.arg_idx() {
								if arguments[idx].is_none() {
									arguments[idx] = Some("u32");
								}
							}
							if let Some(idx) = rt.arg_idx() {
								if arguments[idx].is_none() {
									arguments[idx] = Some("u32");
								}
							}
						},

						Basic(
							Sll { rd, rt, .. } |
							Srl { rd, rt, .. } |
							Sra { rd, rt, .. } |
							MfcN { rt, rd, .. } |
							CfcN { rt, rd, .. } |
							MtcN { rt, rd, .. } |
							CtcN { rt, rd, .. },
						) => {
							if let Some(idx) = rd.arg_idx() {
								if arguments[idx].is_none() {
									arguments[idx] = Some("u32");
								}
							}
							if let Some(idx) = rt.arg_idx() {
								if arguments[idx].is_none() {
									arguments[idx] = Some("u32");
								}
							}
						},

						Basic(Jalr { rd, rs }) => {
							if let Some(idx) = rd.arg_idx() {
								if arguments[idx].is_none() {
									arguments[idx] = Some("u32");
								}
							}
							if let Some(idx) = rs.arg_idx() {
								if arguments[idx].is_none() {
									arguments[idx] = Some("*fn()");
								}
							}
						},

						Basic(Lui { rt, .. }) => {
							if let Some(idx) = rt.arg_idx() {
								if arguments[idx].is_none() {
									arguments[idx] = Some("u32");
								}
							}
						},

						Basic(Mfhi { rd } | Mflo { rd }) => {
							if let Some(idx) = rd.arg_idx() {
								if arguments[idx].is_none() {
									arguments[idx] = Some("u32");
								}
							}
						},

						Basic(
							Bltz { rs, .. } |
							Bgez { rs, .. } |
							Bgtz { rs, .. } |
							Blez { rs, .. } |
							Bltzal { rs, .. } |
							Bgezal { rs, .. } |
							Jr { rs } |
							Mthi { rs } |
							Mtlo { rs },
						) => {
							if let Some(idx) = rs.arg_idx() {
								if arguments[idx].is_none() {
									arguments[idx] = Some("u32");
								}
							}
						},

						Pseudo(MovReg { rx, ry }) => {
							if let Some(idx) = rx.arg_idx() {
								if arguments[idx].is_none() {
									arguments[idx] = Some("u32");
								}
							}
							if let Some(idx) = ry.arg_idx() {
								if arguments[idx].is_none() {
									arguments[idx] = Some("u32");
								}
							}
						},

						Pseudo(La { rx, .. } | Li32 { rx, .. } | LiU16 { rx, .. } | LiI16 { rx, .. } | LiUpper16 { rx, .. }) => {
							if let Some(idx) = rx.arg_idx() {
								if arguments[idx].is_none() {
									arguments[idx] = Some("*u32");
								}
							}
						},

						Pseudo(
							AddAssign { rx, rt } |
							AdduAssign { rx, rt } |
							SubAssign { rx, rt } |
							SubuAssign { rx, rt } |
							AndAssign { rx, rt } |
							OrAssign { rx, rt } |
							XorAssign { rx, rt } |
							NorAssign { rx, rt },
						) => {
							if let Some(idx) = rx.arg_idx() {
								if arguments[idx].is_none() {
									arguments[idx] = Some("u32");
								}
							}
							if let Some(idx) = rt.arg_idx() {
								if arguments[idx].is_none() {
									arguments[idx] = Some("u32");
								}
							}
						},

						Pseudo(
							AddiAssign { rx, .. } |
							AddiuAssign { rx, .. } |
							AndiAssign { rx, .. } |
							OriAssign { rx, .. } |
							XoriAssign { rx, .. } |
							SllAssign { rx, .. } |
							SrlAssign { rx, .. } |
							SraAssign { rx, .. } |
							SubiAssign { rx, .. } |
							SubiuAssign { rx, .. },
						) => {
							if let Some(idx) = rx.arg_idx() {
								if arguments[idx].is_none() {
									arguments[idx] = Some("u32");
								}
							}
						},

						Pseudo(SllvAssign { rx, rs } | SrlvAssign { rx, rs } | SravAssign { rx, rs }) => {
							if let Some(idx) = rx.arg_idx() {
								if arguments[idx].is_none() {
									arguments[idx] = Some("u32");
								}
							}
							if let Some(idx) = rs.arg_idx() {
								if arguments[idx].is_none() {
									arguments[idx] = Some("u32");
								}
							}
						},

						Pseudo(JalrRa { rx } | Beqz { rx, .. } | Bnez { rx, .. }) => {
							if let Some(idx) = rx.arg_idx() {
								if arguments[idx].is_none() {
									arguments[idx] = Some("fn()");
								}
							}
						},

						_ => (),
					}
				}

				#[rustfmt::skip]
				let signature = match arguments {
					[None   , None   , None   , None   ] => String::new(),
					[Some(a), None   , None   , None   ] => format!("fn(a: {a})"),
					[a      , Some(b), None   , None   ] => format!("fn(a: { }, b: {b})"               , a.unwrap_or("???")),
					[a      , b      , Some(c), None   ] => format!("fn(a: { }, b: { }, c: {c})"       , a.unwrap_or("???"), b.unwrap_or("???")),
					[a      , b      , c      , Some(d)] => format!("fn(a: { }, b: { }, c: { } d: {d})", a.unwrap_or("???"), b.unwrap_or("???"), c.unwrap_or("???")),
				};

				Func {
					name: format!("func_{idx}"),
					signature,
					desc: String::new(),
					comments: HashMap::new(),
					labels,
					start_pos: func_pos,
					end_pos,
				}
			})
			.collect()
		*/
		todo!()
	}
}
