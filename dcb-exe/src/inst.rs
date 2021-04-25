#![doc(include = "inst.md")]

// Modules
pub mod basic;
pub mod directive;
pub mod error;
pub mod fmt;
pub mod iter;
pub mod parse;
pub mod pseudo;
pub mod reg;
pub mod size;
pub mod target;

// Exports
pub use directive::Directive;
pub use error::{DecodeError, FromParsedError};
pub use fmt::{InstFmt, InstTargetFmt};
pub use iter::ParseIter;
pub use reg::Register;
pub use size::InstSize;
pub use target::InstTarget;

// Imports
use self::{
	basic::{Decodable as _, Encodable as _},
	parse::{ParsedArg, ParsedInst},
	pseudo::{Decodable as _, Encodable as _},
};
use crate::{DataTable, FuncTable, Pos};
use ascii::AsciiStr;
use int_conv::Signed;
use std::{
	borrow::Borrow,
	collections::HashMap,
	convert::{TryFrom, TryInto},
	io::{self, Write},
	ops::{Add, Deref, Div, Sub},
};

/// An assembler instruction.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[derive(derive_more::TryInto)]
pub enum Inst<'a> {
	/// A basic instruction
	Basic(basic::Inst),

	/// A pseudo instruction
	Pseudo(pseudo::Inst),

	/// A directive
	Directive(Directive<'a>),
}

impl<'a> Inst<'a> {
	/// Decodes an instruction from bytes and it's position.
	pub fn decode(pos: Pos, bytes: &'a [u8], data_table: &'a DataTable, func_table: &'a FuncTable) -> Result<Self, DecodeError<'a>> {
		// If `bytes` is empty, return Err
		if bytes.is_empty() {
			return Err(DecodeError::NoBytes);
		}

		// If we're contained in some data, check it's type so we can read it
		if let Some(data) = data_table.get_containing(pos) {
			return Directive::decode_with_data(pos, bytes, data.ty(), data.start_pos())
				.map(Self::Directive)
				.map_err(|err| DecodeError::InvalidDataLocation { data, err });
		}

		// TODO: Check functions

		// If we're not aligned to a word, decode a directive
		if !pos.is_word_aligned() {
			let directive = Directive::decode(pos, bytes).ok_or(DecodeError::NoBytes)?;
			return Ok(Self::Directive(directive));
		}

		// Else make the instruction iterator
		// Note: We fuse it to make sure that pseudo instructions don't try to skip
		//       invalid instructions.
		let mut insts = bytes
			.array_chunks::<4>()
			.copied()
			.map(u32::from_ne_bytes)
			.map_while(basic::Inst::decode)
			.fuse();

		// Try to decode a pseudo-instruction
		if let Some(inst) = pseudo::Inst::decode(insts.clone()) {
			// Then check if any function labels intersect it
			// Note: Intersecting at the beginning is fine
			let inst_range = (pos + 1u32)..(pos + inst.size());
			match func_table.range(..=inst_range.end).next_back() {
				// If any do, don't return the instruction
				Some(func) if func.labels.range(inst_range).next().is_some() => (),

				// Else return it
				_ => return Ok(Self::Pseudo(inst)),
			}
		}

		// Else try to decode it as an basic instruction
		if let Some(inst) = insts.next() {
			return Ok(Self::Basic(inst));
		}

		// Else read it as a directive
		Directive::decode(pos, bytes).map(Self::Directive).ok_or(DecodeError::NoBytes)
	}

	/// Writes this instruction
	pub fn write(&self, f: &mut impl Write) -> Result<(), io::Error> {
		match self {
			Inst::Basic(inst) => {
				f.write_all(&inst.encode().to_le_bytes())?;
			},
			Inst::Pseudo(inst) => {
				for inst in inst.encode() {
					f.write_all(&inst.encode().to_le_bytes())?;
				}
			},
			Inst::Directive(directive) => directive.write(f)?,
		}

		Ok(())
	}

	/// Get an instruction's size by it's parsed form and position
	///
	/// Note: This function might not report errors with `inst`, if they happen, such
	///       as wrong number of arguments, unless necessary to get it's size.
	#[allow(clippy::too_many_lines)] // TODO: Refactor?
	#[allow(clippy::match_same_arms)] // Too much work to refactor more currently
	pub fn size_from_parsed(inst: &'a ParsedInst, _pos: Pos) -> Result<u32, FromParsedError> {
		let mnemonic = inst.mnemonic.as_str();
		let args = inst.args.as_slice();

		let inst_size = match (mnemonic, args) {
			("dw", _) => 4,
			("dh", _) => 2,
			("db", _) => 1,
			(".ascii", [ParsedArg::String(ref s)]) => (s.len() + (4 - s.len() % 4)).try_into()?,
			("nop", [ParsedArg::Literal(len)]) => (4 * len).try_into()?,
			("nop", []) => 4,
			("li", [_, ParsedArg::Literal(value)]) => match (u16::try_from(*value), i16::try_from(*value)) {
				(Ok(_), _) | (_, Ok(_)) => 4,
				_ => 8,
			},
			("la", _) => 8,

			(
				"sb" | "sh" | "swl" | "sw" | "swr" | "lb" | "lh" | "lwl" | "lw" | "lbu" | "lhu" | "lwr",
				[ParsedArg::Register(_), ParsedArg::RegisterOffset { .. }],
			) => 4,
			(
				"sb" | "sh" | "swl" | "sw" | "swr" | "lb" | "lh" | "lwl" | "lw" | "lbu" | "lhu" | "lwr",
				[ParsedArg::Register(_), ParsedArg::Literal(_) | ParsedArg::Label(_) | ParsedArg::LabelOffset { .. }],
			) => 8,

			// Jump immediate
			(
				"move" | "addi" | "addiu" | "slti" | "sltiu" | "andi" | "ori" | "xori" | "add" | "addu" | "sub" | "subu" | "and" | "or" | "xor" |
				"nor" | "slt" | "sltu" | "sll" | "srl" | "sra" | "sllv" | "srlv" | "srav" | "j" | "jal" | "jr" | "jalr" | "b" | "beqz" | "beq" |
				"bnez" | "bne" | "blez" | "bgtz" | "bltz" | "bgez" | "bltzal" | "bgezal" | "lui" | "cop0" | "cop1" | "cop2" | "cop3" | "mfc0" |
				"mfc1" | "mfc2" | "mfc3" | "cfc0" | "cfc1" | "cfc2" | "cfc3" | "mtc0" | "mtc1" | "mtc2" | "mtc3" | "ctc0" | "ctc1" | "ctc2" |
				"ctc3" | "lwc0" | "lwc1" | "lwc2" | "lwc3" | "swc0" | "swc1" | "swc2" | "swc3" | "mflo" | "mfhi" | "mtlo" | "mthi" | "mult" |
				"multu" | "div" | "divu" | "break" | "sys",
				_,
			) => 4,

			_ => return Err(FromParsedError::UnknownMnemonic),
		};

		Ok(inst_size)
	}

	/// Creates an instruction from a parsed instruction
	#[allow(clippy::too_many_lines)] // TODO: Refactor?
	pub fn from_parsed(inst: &'a ParsedInst, pos: Pos, labels_by_name: &HashMap<LabelName, Pos>) -> Result<Self, FromParsedError> {
		let mnemonic = inst.mnemonic.as_str();
		let args = inst.args.as_slice();

		// Helper that converts a label to a target
		let label_to_target = |label: &str| {
			labels_by_name
				.get(label)
				.copied()
				.ok_or_else(|| FromParsedError::UnknownLabel(label.to_owned()))
		};

		// Helper that converts a label to an offset
		let label_to_offset = |label: &str, offset: i64| -> Result<i16, FromParsedError> {
			label_to_target(label)?
				.sub(pos)
				.add(offset)
				.div(4)
				.sub(1)
				.try_into()
				.map_err(FromParsedError::RelativeJumpTooFar)
		};

		let inst = match mnemonic {
			// Directives
			"dw" | "dh" | "db" | ".ascii" => {
				// Get the argument, we only support single arguments
				let arg: &'a ParsedArg = match args {
					[arg] => arg,
					_ => return Err(FromParsedError::InvalidArguments),
				};

				// Then get the directive itself
				// TODO: Allow `dw`s to have negative numbers by casting them?
				let directive = match mnemonic {
					"dw" => match arg {
						// If it's a label, get the label's address
						ParsedArg::Label(label) => labels_by_name
							.get(label)
							.map(|&Pos(pos)| Directive::Dw(pos))
							.ok_or_else(|| FromParsedError::UnknownLabel(label.clone()))?,
						ParsedArg::LabelOffset { label, offset } => labels_by_name
							.get(label)
							.map(|pos| Ok::<_, FromParsedError>(pos + u32::try_from(*offset)?))
							.ok_or_else(|| FromParsedError::UnknownLabel(label.clone()))?
							.map(|Pos(pos)| Directive::Dw(pos))?,
						&ParsedArg::Literal(value) => Directive::Dw(value.try_into()?),

						_ => return Err(FromParsedError::InvalidArguments),
					},
					"dh" => Directive::Dh(arg.as_literal().ok_or(FromParsedError::InvalidArguments)?.try_into()?),
					"db" => Directive::Db(arg.as_literal().ok_or(FromParsedError::InvalidArguments)?.try_into()?),
					".ascii" => arg
						.as_string()
						.map(AsciiStr::from_ascii)
						.ok_or(FromParsedError::InvalidArguments)?
						.map(Directive::Ascii)
						.map_err(FromParsedError::StringNonAscii)?,
					_ => unreachable!(),
				};

				// And return it
				Self::Directive(directive)
			},

			// Nop
			"nop" => match *args {
				[ParsedArg::Literal(len)] => Self::Pseudo(pseudo::Inst::Nop(pseudo::nop::Inst { len: len.try_into()? })),
				[] => Self::Pseudo(pseudo::Inst::Nop(pseudo::nop::Inst { len: 1 })),
				_ => return Err(FromParsedError::InvalidArguments),
			},

			// Move
			"move" => match *args {
				[ParsedArg::Register(dst), ParsedArg::Register(src)] => Self::Pseudo(pseudo::Inst::MoveReg(pseudo::move_reg::Inst { dst, src })),
				_ => return Err(FromParsedError::InvalidArguments),
			},

			// Load immediate
			"li" => {
				// Note: No labels for `li`
				let (reg, value) = match *args {
					[ParsedArg::Register(reg), ParsedArg::Literal(value)] => (reg, value),
					_ => return Err(FromParsedError::InvalidArguments),
				};

				// Try to convert it to a `i16`, then `u16`, then `u32`.
				// Note: It seems it is preferred to try `i16` first.
				let kind = if let Ok(value) = value.try_into() {
					pseudo::load_imm::Kind::HalfWordSigned(value)
				} else if let Ok(value) = value.try_into() {
					pseudo::load_imm::Kind::HalfWordUnsigned(value)
				} else {
					pseudo::load_imm::Kind::Word(value.try_into()?)
				};

				Self::Pseudo(pseudo::Inst::LoadImm(pseudo::load_imm::Inst { dst: reg, kind }))
			},

			// Load address
			"la" => {
				let (dst, target) = match *args {
					[ParsedArg::Register(dst), ParsedArg::Literal(value)] => (dst, Pos(value.try_into()?)),
					[ParsedArg::Register(dst), ParsedArg::Label(ref label)] => (dst, label_to_target(label)?),
					[ParsedArg::Register(dst), ParsedArg::LabelOffset { ref label, offset }] => {
						(dst, label_to_target(label)? + i32::try_from(offset)?)
					},

					_ => return Err(FromParsedError::InvalidArguments),
				};

				Self::Pseudo(pseudo::Inst::LoadImm(pseudo::load_imm::Inst {
					dst,
					kind: pseudo::load_imm::Kind::Address(target),
				}))
			},

			// Alu Immediate
			"addi" | "addiu" | "slti" | "sltiu" | "andi" | "ori" | "xori" => {
				let (reg1, reg2, lit) = match *args {
					[ParsedArg::Register(reg), ParsedArg::Literal(lit)] => (reg, reg, lit),
					[ParsedArg::Register(reg1), ParsedArg::Register(reg2), ParsedArg::Literal(lit)] => (reg1, reg2, lit),
					_ => return Err(FromParsedError::InvalidArguments),
				};

				Self::Basic(basic::Inst::Alu(basic::alu::Inst::Imm(basic::alu::imm::Inst {
					dst:  reg1,
					lhs:  reg2,
					kind: match mnemonic {
						"addi" => basic::alu::imm::Kind::Add(lit.try_into()?),
						"addiu" => basic::alu::imm::Kind::AddUnsigned(lit.try_into()?),
						"slti" => basic::alu::imm::Kind::SetLessThan(lit.try_into()?),
						"sltiu" => basic::alu::imm::Kind::SetLessThanUnsigned(lit.try_into()?),
						"andi" => basic::alu::imm::Kind::And(lit.try_into()?),
						"ori" => basic::alu::imm::Kind::Or(lit.try_into()?),
						"xori" => basic::alu::imm::Kind::Xor(lit.try_into()?),
						_ => unreachable!(),
					},
				})))
			},

			// Alu register
			"add" | "addu" | "sub" | "subu" | "and" | "or" | "xor" | "nor" | "slt" | "sltu" => {
				let (reg1, reg2, reg3) = match *args {
					[ParsedArg::Register(reg1), ParsedArg::Register(reg2)] => (reg1, reg1, reg2),
					[ParsedArg::Register(reg1), ParsedArg::Register(reg2), ParsedArg::Register(reg3)] => (reg1, reg2, reg3),
					_ => return Err(FromParsedError::InvalidArguments),
				};

				Self::Basic(basic::Inst::Alu(basic::alu::Inst::Reg(basic::alu::reg::Inst {
					dst:  reg1,
					lhs:  reg2,
					rhs:  reg3,
					kind: match mnemonic {
						"add" => basic::alu::reg::Kind::Add,
						"addu" => basic::alu::reg::Kind::AddUnsigned,
						"sub" => basic::alu::reg::Kind::Sub,
						"subu" => basic::alu::reg::Kind::SubUnsigned,
						"and" => basic::alu::reg::Kind::And,
						"or" => basic::alu::reg::Kind::Or,
						"xor" => basic::alu::reg::Kind::Xor,
						"nor" => basic::alu::reg::Kind::Nor,
						"slt" => basic::alu::reg::Kind::SetLessThan,
						"sltu" => basic::alu::reg::Kind::SetLessThanUnsigned,
						_ => unreachable!(),
					},
				})))
			},

			// Shift Immediate
			"sll" | "srl" | "sra" => {
				let (reg1, reg2, lit) = match *args {
					[ParsedArg::Register(reg), ParsedArg::Literal(lit)] => (reg, reg, lit),
					[ParsedArg::Register(reg1), ParsedArg::Register(reg2), ParsedArg::Literal(lit)] => (reg1, reg2, lit),
					_ => return Err(FromParsedError::InvalidArguments),
				};

				Self::Basic(basic::Inst::Shift(basic::shift::Inst::Imm(basic::shift::imm::Inst {
					dst:  reg1,
					lhs:  reg2,
					rhs:  lit.try_into()?,
					kind: match mnemonic {
						"sll" => basic::shift::imm::Kind::LeftLogical,
						"srl" => basic::shift::imm::Kind::RightLogical,
						"sra" => basic::shift::imm::Kind::RightArithmetic,
						_ => unreachable!(),
					},
				})))
			},

			// Shift register
			"sllv" | "srlv" | "srav" => {
				let (reg1, reg2, reg3) = match *args {
					[ParsedArg::Register(reg1), ParsedArg::Register(reg2)] => (reg1, reg1, reg2),
					[ParsedArg::Register(reg1), ParsedArg::Register(reg2), ParsedArg::Register(reg3)] => (reg1, reg2, reg3),
					_ => return Err(FromParsedError::InvalidArguments),
				};

				Self::Basic(basic::Inst::Shift(basic::shift::Inst::Reg(basic::shift::reg::Inst {
					dst:  reg1,
					lhs:  reg2,
					rhs:  reg3,
					kind: match mnemonic {
						"sllv" => basic::shift::reg::Kind::LeftLogical,
						"srlv" => basic::shift::reg::Kind::RightLogical,
						"srav" => basic::shift::reg::Kind::RightArithmetic,
						_ => unreachable!(),
					},
				})))
			},

			// Store / Load
			"sb" | "sh" | "swl" | "sw" | "swr" | "lb" | "lh" | "lwl" | "lw" | "lbu" | "lhu" | "lwr" => {
				let (reg1, reg2_offset, target) = match *args {
					[ParsedArg::Register(reg1), ParsedArg::RegisterOffset { register: reg2, offset }] => {
						(reg1, Some((reg2, offset.try_into()?)), None)
					},
					[ParsedArg::Register(reg), ParsedArg::Literal(pos)] => (reg, None, Some(Pos(pos.try_into()?))),
					[ParsedArg::Register(reg), ParsedArg::Label(ref label)] => (reg, None, Some(label_to_target(label)?)),
					[ParsedArg::Register(reg), ParsedArg::LabelOffset { ref label, offset }] => {
						(reg, None, Some(label_to_target(label)? + i32::try_from(offset)?))
					},
					_ => return Err(FromParsedError::InvalidArguments),
				};

				match (mnemonic, reg2_offset, target) {
					("sb" | "sh" | "swl" | "sw" | "swr", Some((reg2, offset)), None) => Self::Basic(basic::Inst::Store(basic::store::Inst {
						value: reg1,
						addr: reg2,
						offset,
						kind: match mnemonic {
							"sb" => basic::store::Kind::Byte,
							"sh" => basic::store::Kind::HalfWord,
							"swl" => basic::store::Kind::WordLeft,
							"sw" => basic::store::Kind::Word,
							"swr" => basic::store::Kind::WordRight,
							_ => unreachable!(),
						},
					})),
					("sb" | "sh" | "swl" | "sw" | "swr", None, Some(target)) => Self::Pseudo(pseudo::Inst::Store(pseudo::store::Inst {
						value: reg1,
						target,
						kind: match mnemonic {
							"sb" => basic::store::Kind::Byte,
							"sh" => basic::store::Kind::HalfWord,
							"swl" => basic::store::Kind::WordLeft,
							"sw" => basic::store::Kind::Word,
							"swr" => basic::store::Kind::WordRight,
							_ => unreachable!(),
						},
					})),
					("lb" | "lh" | "lwl" | "lw" | "lbu" | "lhu" | "lwr", Some((reg2, offset)), None) => {
						Self::Basic(basic::Inst::Load(basic::load::Inst {
							value: reg1,
							addr: reg2,
							offset,
							kind: match mnemonic {
								"lb" => basic::load::Kind::Byte,
								"lh" => basic::load::Kind::HalfWord,
								"lwl" => basic::load::Kind::WordLeft,
								"lw" => basic::load::Kind::Word,
								"lbu" => basic::load::Kind::ByteUnsigned,
								"lhu" => basic::load::Kind::HalfWordUnsigned,
								"lwr" => basic::load::Kind::WordRight,
								_ => unreachable!(),
							},
						}))
					},
					("lb" | "lh" | "lwl" | "lw" | "lbu" | "lhu" | "lwr", None, Some(target)) => {
						Self::Pseudo(pseudo::Inst::Load(pseudo::load::Inst {
							value: reg1,
							target,
							kind: match mnemonic {
								"lb" => basic::load::Kind::Byte,
								"lh" => basic::load::Kind::HalfWord,
								"lwl" => basic::load::Kind::WordLeft,
								"lw" => basic::load::Kind::Word,
								"lbu" => basic::load::Kind::ByteUnsigned,
								"lhu" => basic::load::Kind::HalfWordUnsigned,
								"lwr" => basic::load::Kind::WordRight,
								_ => unreachable!(),
							},
						}))
					},
					_ => unreachable!(),
				}
			},

			// Jump immediate
			"j" | "jal" => {
				let target = match *args {
					[ParsedArg::Literal(pos)] => Pos(pos.try_into()?),
					[ParsedArg::Label(ref label)] => label_to_target(label)?,
					[ParsedArg::LabelOffset { ref label, offset }] => label_to_target(label)? + i32::try_from(offset)?,
					_ => return Err(FromParsedError::InvalidArguments),
				};

				Self::Basic(basic::Inst::Jmp(basic::jmp::Inst::Imm(basic::jmp::imm::Inst {
					imm:  (target.0 & 0x0fffffff) / 4,
					kind: match mnemonic {
						"j" => basic::jmp::imm::Kind::Jump,
						"jal" => basic::jmp::imm::Kind::JumpLink,
						_ => unreachable!(),
					},
				})))
			},

			// Jump register
			"jr" | "jalr" => {
				let (target, link) = match *args {
					[ParsedArg::Register(target)] => (target, None),
					[ParsedArg::Register(target), ParsedArg::Register(link)] => (target, Some(link)),
					_ => return Err(FromParsedError::InvalidArguments),
				};

				Self::Basic(basic::Inst::Jmp(basic::jmp::Inst::Reg(basic::jmp::reg::Inst {
					target,
					kind: match (mnemonic, link) {
						("jr", None) => basic::jmp::reg::Kind::Jump,
						("jalr", None) => basic::jmp::reg::Kind::JumpLink(Register::Ra),
						("jalr", Some(link)) => basic::jmp::reg::Kind::JumpLink(link),
						_ => return Err(FromParsedError::InvalidArguments),
					},
				})))
			},

			// Conditionals
			"b" | "beqz" | "beq" | "bnez" | "bne" | "blez" | "bgtz" | "bltz" | "bgez" | "bltzal" | "bgezal" => {
				// Get all args
				// Note: Literals are absolute
				let (reg1, reg2, offset) = match *args {
					// <reg1> <reg2> <target>
					[ParsedArg::Register(reg1), ParsedArg::Register(reg2), ParsedArg::Literal(target)] => (
						Some(reg1),
						Some(reg2),
						u32::try_from(target)?.wrapping_sub(pos.0).as_signed().div(4i32).sub(1i32).try_into()?,
					),
					[ParsedArg::Register(reg1), ParsedArg::Register(reg2), ParsedArg::Label(ref label)] => {
						(Some(reg1), Some(reg2), label_to_offset(label, 0)?)
					},
					[ParsedArg::Register(reg1), ParsedArg::Register(reg2), ParsedArg::LabelOffset { ref label, offset }] => {
						(Some(reg1), Some(reg2), label_to_offset(label, offset)?)
					},

					// <reg> <target>
					[ParsedArg::Register(reg1), ParsedArg::Literal(target)] => (
						Some(reg1),
						None,
						u32::try_from(target)?.wrapping_sub(pos.0).as_signed().div(4i32).sub(1i32).try_into()?,
					),
					[ParsedArg::Register(reg1), ParsedArg::Label(ref label)] => (Some(reg1), None, label_to_offset(label, 0)?),
					[ParsedArg::Register(reg1), ParsedArg::LabelOffset { ref label, offset }] => (Some(reg1), None, label_to_offset(label, offset)?),

					// <target>
					[ParsedArg::Literal(target)] => (
						None,
						None,
						u32::try_from(target)?.wrapping_sub(pos.0).as_signed().div(4i32).sub(1i32).try_into()?,
					),
					[ParsedArg::Label(ref label)] => (None, None, label_to_offset(label, 0)?),
					[ParsedArg::LabelOffset { ref label, offset }] => (None, None, label_to_offset(label, offset)?),
					_ => return Err(FromParsedError::InvalidArguments),
				};

				match (mnemonic, reg1, reg2) {
					("b", None, None) => Self::Basic(basic::Inst::Cond(basic::cond::Inst {
						arg: Register::Zr,
						offset,
						kind: basic::cond::Kind::Equal(Register::Zr),
					})),
					("beqz", Some(arg), None) => Self::Basic(basic::Inst::Cond(basic::cond::Inst {
						arg,
						offset,
						kind: basic::cond::Kind::Equal(Register::Zr),
					})),
					("bnez", Some(arg), None) => Self::Basic(basic::Inst::Cond(basic::cond::Inst {
						arg,
						offset,
						kind: basic::cond::Kind::NotEqual(Register::Zr),
					})),
					("beq", Some(arg), Some(other)) => Self::Basic(basic::Inst::Cond(basic::cond::Inst {
						arg,
						offset,
						kind: basic::cond::Kind::Equal(other),
					})),
					("bne", Some(arg), Some(other)) => Self::Basic(basic::Inst::Cond(basic::cond::Inst {
						arg,
						offset,
						kind: basic::cond::Kind::NotEqual(other),
					})),
					("blez", Some(arg), None) => Self::Basic(basic::Inst::Cond(basic::cond::Inst {
						arg,
						offset,
						kind: basic::cond::Kind::LessOrEqualZero,
					})),
					("bgtz", Some(arg), None) => Self::Basic(basic::Inst::Cond(basic::cond::Inst {
						arg,
						offset,
						kind: basic::cond::Kind::GreaterThanZero,
					})),
					("bltz", Some(arg), None) => Self::Basic(basic::Inst::Cond(basic::cond::Inst {
						arg,
						offset,
						kind: basic::cond::Kind::LessThanZero,
					})),
					("bgez", Some(arg), None) => Self::Basic(basic::Inst::Cond(basic::cond::Inst {
						arg,
						offset,
						kind: basic::cond::Kind::GreaterOrEqualZero,
					})),
					("bltzal", Some(arg), None) => Self::Basic(basic::Inst::Cond(basic::cond::Inst {
						arg,
						offset,
						kind: basic::cond::Kind::LessThanZeroLink,
					})),
					("bgezal", Some(arg), None) => Self::Basic(basic::Inst::Cond(basic::cond::Inst {
						arg,
						offset,
						kind: basic::cond::Kind::GreaterOrEqualZeroLink,
					})),

					(_, None, Some(_)) => unreachable!(),

					_ => return Err(FromParsedError::InvalidArguments),
				}
			},

			// Lui
			"lui" => match *args {
				[ParsedArg::Register(dst), ParsedArg::Literal(value)] => Self::Basic(basic::Inst::Lui(basic::lui::Inst {
					dst,
					value: value.try_into()?,
				})),
				_ => return Err(FromParsedError::InvalidArguments),
			},

			// Co-processor
			"cop0" | "cop1" | "cop2" | "cop3" => {
				let n = mnemonic[3..].parse().expect("Unable to parse 0..=3");
				let imm = match *args {
					[ParsedArg::Literal(imm)] => imm.try_into()?,
					_ => return Err(FromParsedError::InvalidArguments),
				};

				Self::Basic(basic::Inst::Co(basic::co::Inst {
					n,
					kind: basic::co::Kind::CopN { imm },
				}))
			},
			"mfc0" | "mfc1" | "mfc2" | "mfc3" | "cfc0" | "cfc1" | "cfc2" | "cfc3" | "mtc0" | "mtc1" | "mtc2" | "mtc3" | "ctc0" | "ctc1" |
			"ctc2" | "ctc3" => {
				let n = mnemonic[3..].parse().expect("Unable to parse 0..=3");
				let (reg, imm) = match *args {
					[ParsedArg::Register(dst), ParsedArg::Literal(src)] => (dst, src.try_into()?),
					_ => return Err(FromParsedError::InvalidArguments),
				};

				let kind = match &mnemonic[0..=0] {
					"m" => basic::co::RegisterKind::Data,
					"c" => basic::co::RegisterKind::Control,
					_ => unreachable!(),
				};

				match &mnemonic[1..=1] {
					"f" => Self::Basic(basic::Inst::Co(basic::co::Inst {
						n,
						kind: basic::co::Kind::MoveFrom { dst: reg, src: imm, kind },
					})),
					"t" => Self::Basic(basic::Inst::Co(basic::co::Inst {
						n,
						kind: basic::co::Kind::MoveTo { dst: imm, src: reg, kind },
					})),
					_ => unreachable!(),
				}
			},
			"lwc0" | "lwc1" | "lwc2" | "lwc3" | "swc0" | "swc1" | "swc2" | "swc3" => {
				let n = mnemonic[3..].parse().expect("Unable to parse 0..=3");
				let (dst, src, offset) = match *args {
					[ParsedArg::Literal(dst), ParsedArg::RegisterOffset { register: src, offset }] => (dst.try_into()?, src, offset.try_into()?),
					_ => return Err(FromParsedError::InvalidArguments),
				};

				match &mnemonic[0..=0] {
					"l" => Self::Basic(basic::Inst::Co(basic::co::Inst {
						n,
						kind: basic::co::Kind::Load { dst, src, offset },
					})),
					"s" => Self::Basic(basic::Inst::Co(basic::co::Inst {
						n,
						kind: basic::co::Kind::Store { dst, src, offset },
					})),
					_ => unreachable!(),
				}
			},

			// Mult move
			"mflo" | "mfhi" | "mtlo" | "mthi" => {
				let reg = match *args {
					[ParsedArg::Register(reg)] => reg,
					_ => return Err(FromParsedError::InvalidArguments),
				};

				let mult_reg = match &mnemonic[2..=3] {
					"lo" => basic::mult::MultReg::Lo,
					"hi" => basic::mult::MultReg::Hi,
					_ => unreachable!(),
				};


				match &mnemonic[1..=1] {
					"f" => Self::Basic(basic::Inst::Mult(basic::mult::Inst::MoveFrom { dst: reg, src: mult_reg })),
					"t" => Self::Basic(basic::Inst::Mult(basic::mult::Inst::MoveTo { dst: mult_reg, src: reg })),
					_ => unreachable!(),
				}
			},

			// Mult / Div
			"mult" | "multu" | "div" | "divu" => {
				let (lhs, rhs) = match *args {
					[ParsedArg::Register(lhs), ParsedArg::Register(rhs)] => (lhs, rhs),
					_ => return Err(FromParsedError::InvalidArguments),
				};

				Self::Basic(basic::Inst::Mult(basic::mult::Inst::Mult {
					lhs,
					rhs,
					mode: match mnemonic {
						"divu" | "multu" => basic::mult::MultMode::Unsigned,
						"div" | "mult" => basic::mult::MultMode::Signed,
						_ => unreachable!(),
					},
					kind: match mnemonic {
						"mult" | "multu" => basic::mult::MultKind::Mult,
						"div" | "divu" => basic::mult::MultKind::Div,
						_ => unreachable!(),
					},
				}))
			},

			// Syscalls
			"break" | "sys" => {
				let comment = match *args {
					[ParsedArg::Literal(comment)] => comment.try_into()?,
					_ => return Err(FromParsedError::InvalidArguments),
				};

				Self::Basic(basic::Inst::Sys(basic::sys::Inst {
					comment,
					kind: match mnemonic {
						"break" => basic::sys::Kind::Break,
						"sys" => basic::sys::Kind::Sys,
						_ => return Err(FromParsedError::InvalidArguments),
					},
				}))
			},
			_ => return Err(FromParsedError::UnknownMnemonic),
		};

		Ok(inst)
	}
}

impl<'a> InstSize for Inst<'a> {
	fn size(&self) -> usize {
		match self {
			Inst::Basic(inst) => inst.size(),
			Inst::Pseudo(inst) => inst.size(),
			Inst::Directive(directive) => directive.size(),
		}
	}
}

impl<'a> InstFmt for Inst<'a> {
	fn fmt(&self, pos: Pos, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Self::Basic(inst) => inst.fmt(pos, f),
			Self::Pseudo(inst) => inst.fmt(pos, f),
			Self::Directive(directive) => <Directive as InstFmt>::fmt(directive, pos, f),
		}
	}
}

/// Label
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Label {
	/// Local
	Local {
		/// Global name, '<parent>.<local>'
		name: LabelName,
	},

	/// Global
	Global {
		/// Name
		name: LabelName,
	},
}

impl Label {
	/// Returns the name of this label
	#[must_use]
	pub const fn name(&self) -> &LabelName {
		match self {
			Label::Local { name } | Label::Global { name } => name,
		}
	}

	/// Returns this label as local
	#[must_use]
	pub const fn as_local(&self) -> Option<&LabelName> {
		match self {
			Self::Local { name } => Some(name),
			_ => None,
		}
	}

	/// Returns this label as global
	#[must_use]
	pub const fn as_global(&self) -> Option<&LabelName> {
		match self {
			Self::Global { name } => Some(name),
			_ => None,
		}
	}
}

/// Label name
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Debug)]
pub struct LabelName(pub String);

impl Deref for LabelName {
	type Target = String;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl Borrow<String> for LabelName {
	fn borrow(&self) -> &String {
		&self.0
	}
}

impl Borrow<str> for LabelName {
	fn borrow(&self) -> &str {
		&self.0
	}
}
