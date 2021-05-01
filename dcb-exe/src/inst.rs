#![doc(include = "inst.md")]

// Modules
pub mod basic;
pub mod decode;
pub mod directive;
pub mod error;
pub mod exec;
pub mod fmt;
pub mod parse;
pub mod pseudo;
pub mod reg;
pub mod size;

// Exports
pub use decode::DecodeIter;
pub use directive::Directive;
pub use error::DecodeError;
pub use fmt::{DisplayCtx, InstDisplay, InstFmtArg};
pub use parse::{Parsable, ParseCtx, ParseError};
pub use reg::Register;
pub use size::InstSize;

// Imports
use self::{
	basic::{Decode as _, TryEncode as _},
	parse::LineArg,
	pseudo::{Decodable as _, Encodable as _},
};
use crate::{DataTable, FuncTable, Pos};
use std::{borrow::Borrow, io, ops::Deref};

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
	pub fn decode(
		pos: Pos, bytes: &'a [u8], data_table: &'a DataTable, func_table: &'a FuncTable,
	) -> Result<Self, DecodeError<'a>> {
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
		Directive::decode(pos, bytes)
			.map(Self::Directive)
			.ok_or(DecodeError::NoBytes)
	}

	/// Writes an instruction
	pub fn write(&self, f: &mut impl io::Write) -> Result<(), WriteError> {
		match self {
			Inst::Basic(inst) => f.write_all(&inst.try_encode().map_err(WriteError::EncodeBasic)?.to_le_bytes())?,
			Inst::Pseudo(inst) => {
				for inst in inst.encode() {
					f.write_all(&inst.try_encode().map_err(WriteError::EncodeBasic)?.to_le_bytes())?;
				}
			},
			Inst::Directive(directive) => directive.write(f)?,
		};

		Ok(())
	}
}

impl<'a> Parsable<'a> for Inst<'a> {
	fn parse<Ctx: ?Sized + ParseCtx>(mnemonic: &'a str, args: &'a [LineArg], ctx: &'a Ctx) -> Result<Self, ParseError> {
		#[rustfmt::skip]
		let parsers: &[&dyn Fn() -> Result<Self, ParseError>] = &[
			&|| basic    ::Inst::parse(mnemonic, args, ctx).map(Self::Basic),
			&|| pseudo   ::Inst::parse(mnemonic, args, ctx).map(Self::Pseudo),
			&||       Directive::parse(mnemonic, args, ctx).map(Self::Directive),
		];

		// Try to parse each one one by one.
		// If we get an unknown mnemonic, try the next, else return the error.
		for parser in parsers {
			match parser() {
				Ok(inst) => return Ok(inst),
				Err(ParseError::UnknownMnemonic) => continue,
				Err(err) => return Err(err),
			}
		}

		Err(ParseError::UnknownMnemonic)
	}
}

impl<'a> InstDisplay<'a> for Inst<'a> {
	type Args = impl Iterator<Item = InstFmtArg<'a>>;
	type Mnemonic = impl std::fmt::Display;

	#[auto_enums::auto_enum(Display)]
	#[rustfmt::skip]
	fn mnemonic<Ctx: DisplayCtx>(&'a self, ctx: &Ctx) -> Self::Mnemonic {
		match self {
			Inst::Basic    (inst) => inst.mnemonic(ctx),
			Inst::Pseudo   (inst) => inst.mnemonic(ctx),
			Inst::Directive(inst) => inst.mnemonic(ctx),
		}
	}

	#[auto_enums::auto_enum(Iterator)]
	#[rustfmt::skip]
	fn args<Ctx: DisplayCtx>(&'a self, ctx: &Ctx) -> Self::Args {
		match self {
			Inst::Basic    (inst) => inst.args(ctx),
			Inst::Pseudo   (inst) => inst.args(ctx),
			Inst::Directive(inst) => inst.args(ctx),
		}
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

/// Write error
#[derive(Debug, thiserror::Error)]
pub enum WriteError {
	/// Io
	#[error("Unable to write")]
	Write(#[from] io::Error),

	/// Encode basic
	#[error("Unable to encode `basic` instruction")]
	EncodeBasic(#[source] basic::EncodeError),
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
