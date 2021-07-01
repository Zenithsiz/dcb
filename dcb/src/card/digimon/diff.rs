//! Digimon differences

// Imports
use crate::card::property::{
	ArrowColor, AttackType, CrossMoveEffect, Effect, EffectCondition, Level, Move, Speciality,
};
use zutil::{AsciiStrArr, TryOrEmpty};
use std::ops::Try;

/// Visitor trait for differences between two digimon
pub trait DiffVisitor {
	/// Result type for each visit
	type Result: Try<Output = ()>;

	/// Visits a name difference
	fn visit_name(&mut self, lhs: &AsciiStrArr<0x14>, rhs: &AsciiStrArr<0x14>) -> Self::Result;

	/// Visits a speciality difference
	fn visit_speciality(&mut self, lhs: Speciality, rhs: Speciality) -> Self::Result;

	/// Visits a level difference
	fn visit_level(&mut self, lhs: Level, rhs: Level) -> Self::Result;

	/// Visits an hp difference
	fn visit_hp(&mut self, lhs: u16, rhs: u16) -> Self::Result;

	/// Visits an dp cost difference
	fn visit_dp_cost(&mut self, lhs: u8, rhs: u8) -> Self::Result;

	/// Visits an dp give difference
	fn visit_dp_give(&mut self, lhs: u8, rhs: u8) -> Self::Result;

	/// Visits an move difference
	fn visit_move(&mut self, attack: AttackType, lhs: &Move, rhs: &Move) -> Self::Result;

	/// Visits an cross move effect difference
	fn visit_cross_move_effect(&mut self, lhs: Option<CrossMoveEffect>, rhs: Option<CrossMoveEffect>) -> Self::Result;

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

	/// Speciality
	Speciality(Speciality, Speciality),

	/// Level
	Level(Level, Level),

	/// Hp
	Hp(u16, u16),

	/// DP Cost
	DpCost(u8, u8),

	/// DP Give
	DpGive(u8, u8),

	/// Move
	Move {
		attack: AttackType,
		lhs:    &'a Move,
		rhs:    &'a Move,
	},

	/// Cross move effect
	CrossMoveEffect(Option<CrossMoveEffect>, Option<CrossMoveEffect>),

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

	fn visit_speciality(&mut self, lhs: Speciality, rhs: Speciality) -> Self::Result {
		T::into_try(self(DiffKind::Speciality(lhs, rhs)))
	}

	fn visit_level(&mut self, lhs: Level, rhs: Level) -> Self::Result {
		T::into_try(self(DiffKind::Level(lhs, rhs)))
	}

	fn visit_hp(&mut self, lhs: u16, rhs: u16) -> Self::Result {
		T::into_try(self(DiffKind::Hp(lhs, rhs)))
	}

	fn visit_dp_cost(&mut self, lhs: u8, rhs: u8) -> Self::Result {
		T::into_try(self(DiffKind::DpCost(lhs, rhs)))
	}

	fn visit_dp_give(&mut self, lhs: u8, rhs: u8) -> Self::Result {
		T::into_try(self(DiffKind::DpGive(lhs, rhs)))
	}

	fn visit_move(&mut self, attack: AttackType, lhs: &Move, rhs: &Move) -> Self::Result {
		T::into_try(self(DiffKind::Move { attack, lhs, rhs }))
	}

	fn visit_cross_move_effect(&mut self, lhs: Option<CrossMoveEffect>, rhs: Option<CrossMoveEffect>) -> Self::Result {
		T::into_try(self(DiffKind::CrossMoveEffect(lhs, rhs)))
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
