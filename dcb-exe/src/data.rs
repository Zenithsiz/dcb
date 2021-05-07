//! Executable data locations
//!
//! This module defines the [`Data`] type, which
//! is responsible for storing a location within
//! the executable that represents a certain data
//! type, with associated metadata, such as a name
//! and description.

// Modules
pub mod kind;
pub mod table;
pub mod ty;

// Exports
pub use kind::DataKind;
pub use table::DataTable;
pub use ty::DataType;

// Imports
use crate::{
	inst::{self, basic, pseudo, Inst, Label},
	Pos,
};
use inst::directive::Directive;
use int_conv::SignExtended;
use std::{collections::BTreeSet, ops::Range};

/// A data location.
///
/// Two data locations are considered equal if they
/// share the same position.
///
/// Their relative order first depends on their position.
/// When their positions are equal, the larger one will
/// appear first in the order.
/// This is to implement `specialized` data locations, where
/// a large data location can have multiple data locations inside.
#[derive(Clone, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(derive_more::Display)]
#[display(fmt = "{name} ({ty}) @ {pos}")]
pub struct Data {
	/// Name
	name: String,

	/// Description
	#[serde(default)]
	desc: String,

	/// Start position
	pos: Pos,

	/// Data type
	ty: DataType,

	/// Data kind
	#[serde(default = "DataKind::known")]
	kind: DataKind,
}

impl Data {
	/// Creates a dummy over all of [`Pos`]'s range
	pub(crate) fn dummy() -> Self {
		Self {
			name: String::new(),
			desc: String::new(),
			pos:  Pos(0),
			ty:   DataType::Array {
				ty:  Box::new(DataType::Word),
				len: 0xFFFF_FFFF / 4,
			},
			kind: DataKind::Known,
		}
	}

	/// Returns this data's name
	#[must_use]
	pub fn name(&self) -> &str {
		&self.name
	}

	/// Returns this data's description
	#[must_use]
	pub fn desc(&self) -> &str {
		&self.desc
	}

	/// Returns this data's kind
	#[must_use]
	pub const fn kind(&self) -> DataKind {
		self.kind
	}

	/// Returns the start position of this data as a reference
	#[must_use]
	pub(self) const fn start_pos_ref(&self) -> &Pos {
		&self.pos
	}

	/// Returns the start position of this data
	#[must_use]
	pub const fn start_pos(&self) -> Pos {
		self.pos
	}

	/// Returns the end position of this data
	#[must_use]
	pub fn end_pos(&self) -> Pos {
		self.start_pos() + self.size()
	}

	/// Returns the range of positions of this data
	#[must_use]
	pub fn pos_range(&self) -> Range<Pos> {
		self.start_pos()..self.end_pos()
	}

	/// Checks if this data contains `pos`
	#[must_use]
	pub fn contains_pos(&self, pos: Pos) -> bool {
		self.pos_range().contains(&pos)
	}

	/// Returns this data's type
	#[must_use]
	pub const fn ty(&self) -> &DataType {
		&self.ty
	}

	/// Returns the size, in bytes, of this data
	#[must_use]
	pub fn size(&self) -> usize {
		self.ty.size()
	}

	/// Consumes this data and returns it as a label
	#[must_use]
	#[allow(clippy::missing_const_for_fn)] // False positive
	pub fn into_label(self) -> (Pos, Label) {
		(self.pos, Label::new(self.name))
	}
}

impl Data {
	/// Searches all instructions for references to
	/// executable data using certain heuristics.
	#[must_use]
	pub fn search_instructions<'a>(
		insts_range: Range<Pos>, insts: impl Iterator<Item = (Pos, Inst<'a>)> + Clone,
	) -> Vec<Self> {
		// Get all possible references to data
		let references: BTreeSet<Pos> = insts
			.clone()
			.filter_map(|(pos, inst)| match inst {
				Inst::Basic(
					basic::Inst::Load(basic::load::Inst { offset, .. }) |
					basic::Inst::Store(basic::store::Inst { offset, .. }),
				) => Some(pos + offset.sign_extended::<i32>()),
				Inst::Pseudo(
					pseudo::Inst::LoadImm(pseudo::load_imm::Inst {
						kind: pseudo::load_imm::Kind::Address(Pos(address)) | pseudo::load_imm::Kind::Word(address),
						..
					}) |
					pseudo::Inst::Load(pseudo::load::Inst {
						target: Pos(address), ..
					}) |
					pseudo::Inst::Store(pseudo::store::Inst {
						target: Pos(address), ..
					}),
				) |
				Inst::Directive(Directive::Dw(address)) => Some(Pos(address)),
				_ => None,
			})
			.filter(|pos| insts_range.contains(pos))
			.collect();

		// Then filter the instructions for data locations.
		insts
			// Filter all non-directives
			.filter_map(|(pos, instruction)| match instruction {
				Inst::Directive(directive) if references.contains(&pos) => Some((pos, directive)),
				_ => None,
			})
			.enumerate()
			.map(|(idx, (pos, directive))| {
				match directive {
					Directive::Ascii(string) => Self {
						name: format!("string_{idx}"),
						desc: String::new(),
						pos,
						ty: DataType::AsciiStr { len: string.len() },
						kind: DataKind::Heuristics,
					},
					Directive::Dw(_) => Self {
						name: format!("data_w{idx}"),
						desc: String::new(),
						pos,
						ty: DataType::Word,
						kind: DataKind::Heuristics,
					},
					Directive::Dh(_) => Self {
						name: format!("data_h{idx}"),
						desc: String::new(),
						pos,
						ty: DataType::HalfWord,
						kind: DataKind::Heuristics,
					},
					Directive::Db(_) => Self {
						name: format!("data_b{idx}"),
						desc: String::new(),
						pos,
						ty: DataType::Byte,
						kind: DataKind::Heuristics,
					},
				}
			})
			.collect()
	}
}
