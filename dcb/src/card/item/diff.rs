//! Item differences

// Imports
use crate::card::property::{ArrowColor, Effect, EffectCondition};
use std::ops::Try;
use zutil::{AsciiStrArr, TryOrEmpty};

/// Visitor trait for differences between two items
pub trait DiffVisitor {
	/// Result type for each visit
	type Result: Try<Output = ()>;

	/// Visits a name difference
	fn visit_name(&mut self, lhs: &AsciiStrArr<0x14>, rhs: &AsciiStrArr<0x14>) -> Self::Result;

	/// Visits an effect description difference
	fn visit_effect_description(
		&mut self, idx: usize, lhs: &AsciiStrArr<0x14>, rhs: &AsciiStrArr<0x14>,
	) -> Self::Result;

	/// Visits an effect arrow color difference
	fn visit_effect_arrow_color(&mut self, lhs: Option<ArrowColor>, rhs: Option<ArrowColor>) -> Self::Result;

	/// Visits an effect condition difference
	fn visit_effect_condition(
		&mut self, idx: usize, lhs: Option<EffectCondition>, rhs: Option<EffectCondition>,
	) -> Self::Result;

	/// Visits an effect difference
	fn visit_effect(&mut self, idx: usize, lhs: &Option<Effect>, rhs: &Option<Effect>) -> Self::Result;

	// TODO: Maybe visitors for unknown fields?
}

/// Enum with all diff variants for use with the impl for functions
#[allow(clippy::missing_docs_in_private_items)] // They're obvious by their name
pub enum DiffKind<'a> {
	/// Name
	Name(&'a AsciiStrArr<0x14>, &'a AsciiStrArr<0x14>),

	/// Effect description
	EffectDescription {
		idx: usize,
		lhs: &'a AsciiStrArr<0x14>,
		rhs: &'a AsciiStrArr<0x14>,
	},

	/// Effect arrow color
	EffectArrowColor(Option<ArrowColor>, Option<ArrowColor>),

	/// Effect condition
	EffectCondition {
		idx: usize,
		lhs: Option<EffectCondition>,
		rhs: Option<EffectCondition>,
	},

	/// Effect
	Effect {
		idx: usize,
		lhs: &'a Option<Effect>,
		rhs: &'a Option<Effect>,
	},
}

impl<T: TryOrEmpty, F> DiffVisitor for F
where
	F: for<'a> FnMut(DiffKind<'a>) -> T,
{
	type Result = T::Try;

	fn visit_name(&mut self, lhs: &AsciiStrArr<0x14>, rhs: &AsciiStrArr<0x14>) -> Self::Result {
		T::into_try(self(DiffKind::Name(lhs, rhs)))
	}

	fn visit_effect_description(
		&mut self, idx: usize, lhs: &AsciiStrArr<0x14>, rhs: &AsciiStrArr<0x14>,
	) -> Self::Result {
		T::into_try(self(DiffKind::EffectDescription { idx, lhs, rhs }))
	}

	fn visit_effect_arrow_color(&mut self, lhs: Option<ArrowColor>, rhs: Option<ArrowColor>) -> Self::Result {
		T::into_try(self(DiffKind::EffectArrowColor(lhs, rhs)))
	}

	fn visit_effect_condition(
		&mut self, idx: usize, lhs: Option<EffectCondition>, rhs: Option<EffectCondition>,
	) -> Self::Result {
		T::into_try(self(DiffKind::EffectCondition { idx, lhs, rhs }))
	}

	fn visit_effect(&mut self, idx: usize, lhs: &Option<Effect>, rhs: &Option<Effect>) -> Self::Result {
		T::into_try(self(DiffKind::Effect { idx, lhs, rhs }))
	}
}
