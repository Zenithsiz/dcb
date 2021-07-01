//! Digivolve differences

// Imports
use crate::card::property::DigivolveEffect;
use std::ops::Try;
use zutil::{AsciiStrArr, TryOrEmpty};

/// Visitor trait for differences between two digivolves
pub trait DiffVisitor {
	/// Result type for each visit
	type Result: Try<Output = ()>;

	/// Visits a name difference
	fn visit_name(&mut self, lhs: &AsciiStrArr<0x14>, rhs: &AsciiStrArr<0x14>) -> Self::Result;

	/// Visits an effect description difference
	fn visit_effect_description(
		&mut self, idx: usize, lhs: &AsciiStrArr<0x14>, rhs: &AsciiStrArr<0x14>,
	) -> Self::Result;

	/// Visits an effect difference
	fn visit_effect(&mut self, lhs: DigivolveEffect, rhs: DigivolveEffect) -> Self::Result;
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

	/// Effect
	Effect(DigivolveEffect, DigivolveEffect),
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

	fn visit_effect(&mut self, lhs: DigivolveEffect, rhs: DigivolveEffect) -> Self::Result {
		T::into_try(self(DiffKind::Effect(lhs, rhs)))
	}
}
