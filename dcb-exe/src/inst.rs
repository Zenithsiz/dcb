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
pub use error::{DecodeError, ParseError};
pub use fmt::{InstFmt, InstTargetFmt};
pub use iter::ParseIter;
pub use reg::Register;
pub use size::InstSize;
pub use target::InstTarget;

// Imports
use self::{basic::Decodable as _, parse::LineArg, pseudo::Decodable as _};
use crate::{DataTable, FuncTable, Pos};
use std::{borrow::Borrow, convert::TryInto, ops::Deref};

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

/// Parsing context
pub trait ParseCtx {
	/// Returns the current position
	fn cur_pos(&self) -> Pos;

	/// Returns the position of a label
	fn label_pos(&self, label: &str) -> Option<Pos>;

	/// Retrieves a position from an argument
	fn arg_pos(&self, arg: &LineArg) -> Result<Pos, basic::ParseError> {
		match *arg {
			LineArg::Literal(pos) => pos.try_into().map(Pos).map_err(|_| basic::ParseError::LiteralOutOfRange),
			LineArg::Label(ref label) => self.label_pos(label).ok_or(basic::ParseError::UnknownLabel),
			_ => Err(basic::ParseError::InvalidArguments),
		}
	}

	/// Retrieves a position and offset from an argument
	fn arg_pos_offset(&self, arg: &LineArg) -> Result<(Pos, i64), basic::ParseError> {
		match *arg {
			LineArg::Literal(pos) => pos.try_into().map(|pos| (Pos(pos), 0)).map_err(|_| basic::ParseError::LiteralOutOfRange),
			LineArg::Label(ref label) => self.label_pos(label).map(|pos| (pos, 0)).ok_or(basic::ParseError::UnknownLabel),
			LineArg::LabelOffset { ref label, offset } => self.label_pos(label).map(|pos| (pos, offset)).ok_or(basic::ParseError::UnknownLabel),
			_ => Err(basic::ParseError::InvalidArguments),
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
