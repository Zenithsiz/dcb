//! Diff screen

// Imports
use crate::loaded_game::LoadedGame;
use anyhow::Context;
use dcb::{
	card::{
		digimon, digivolve, item,
		property::{ArrowColor, AttackType, CrossMoveEffect, Effect, EffectCondition},
		Card,
	},
	CardTable,
};
use eframe::egui::{self, Color32};
use std::{collections::BTreeMap, path::PathBuf};
use zutil::{kv_par_iter::ParIterValue, AsciiStrArr, CachedValue, KVParIter, StrContainsCaseInsensitive};

/// Diff screen
pub struct DiffScreen {
	/// Other loaded game we're diffing against
	other_loaded_game: LoadedGame,

	/// Search
	search: String,

	/// Cached changes
	#[allow(clippy::type_complexity)] // It's not very complex, just a function pointer
	cached_changes: Option<CachedValue<TableChanges, fn(&CardTable, &CardTable)>>,
}

impl DiffScreen {
	/// Creates a new diff screen
	pub fn new(other_file_path: PathBuf) -> Result<Self, anyhow::Error> {
		// Load the other loaded game
		let other_loaded_game = LoadedGame::load(other_file_path).context("Unable to load other game")?;

		Ok(Self {
			other_loaded_game,
			search: String::new(),
			cached_changes: None,
		})
	}

	/// Displays this diff screen
	pub fn display(&mut self, ui: &mut egui::Ui, loaded_game: &LoadedGame) {
		let self_ptr = self as *const _;
		let lhs = &loaded_game.card_table;
		let rhs = &self.other_loaded_game.card_table;

		// Try to update the cached value or calculate it if we don't have it
		let changes = CachedValue::new_or_update(&mut self.cached_changes, (lhs, rhs), TableChanges::new);

		// Else display the search
		let search = &mut self.search;
		ui.vertical(|ui| {
			ui.label("Search");
			ui.text_edit_singleline(search);
		});
		ui.separator();

		// Then display all differences
		egui::ScrollArea::auto_sized().show(ui, |ui| {
			egui::Grid::new(self_ptr).striped(false).show(ui, |ui| {
				// Header
				ui.colored_label(Color32::from_rgb(255, 0, 0), "Loaded game");
				ui.colored_label(Color32::from_rgb(255, 0, 0), "Other game");
				ui.end_row();

				for (name, &changes) in changes.card_changes() {
					// If the name doesn't match, return
					if !name.as_str().contains_case_insensitive(&*search) {
						continue;
					}

					match changes {
						CardChanges::OnlyInLhs { id } => {
							ui.heading(name.as_str());
							ui.heading("❌");
							ui.end_row();

							ui.heading(format!("{id}"));
							ui.heading("");
							ui.end_row();
						},
						CardChanges::OnlyInRhs { id } => {
							ui.heading("❌");
							ui.heading(name.as_str());
							ui.end_row();

							ui.heading("");
							ui.heading(format!("{id}"));
							ui.end_row();
						},
						CardChanges::Equal { .. } => {
							ui.heading(name.as_str());
							ui.heading(name.as_str());
							ui.end_row();
						},
						CardChanges::DifferentType { lhs_id, rhs_id } => {
							ui.heading(name.as_str());
							ui.heading(name.as_str());
							ui.end_row();
							ui.label(lhs.cards[lhs_id].ty().as_str());
							ui.label(rhs.cards[rhs_id].ty().as_str());
							ui.end_row();
						},
						CardChanges::DifferentDigimon {
							lhs_id,
							rhs_id,
							changes,
						} => {
							let lhs = &lhs.cards[lhs_id].as_digimon().expect("Id wasn't for a digimon");
							let rhs = &rhs.cards[rhs_id].as_digimon().expect("Id wasn't for a digimon");

							ui.heading(name.as_str());
							ui.heading(name.as_str());
							ui.end_row();

							if changes.speciality {
								ui.label(format!("Speciality: {}", lhs.speciality));
								ui.label(format!("Speciality: {}", rhs.speciality));
								ui.end_row();
							}
							if changes.level {
								ui.label(format!("Level: {}", lhs.level));
								ui.label(format!("Level: {}", rhs.level));
								ui.end_row();
							}
							if changes.hp {
								ui.label(format!("Hp: {}", lhs.hp));
								ui.label(format!("Hp: {}", rhs.hp));
								ui.end_row();
							}
							if changes.dp_cost {
								ui.label(format!("DP: {}", lhs.dp_cost));
								ui.label(format!("DP: {}", rhs.dp_cost));
								ui.end_row();
							}
							if changes.dp_give {
								ui.label(format!("+P: {}", lhs.dp_give));
								ui.label(format!("+P: {}", rhs.dp_give));
								ui.end_row();
							}
							for (attack, lhs_mv, rhs_mv, changed) in [
								(
									AttackType::Circle,
									&lhs.move_circle,
									&rhs.move_circle,
									changes.move_circle,
								),
								(
									AttackType::Triangle,
									&lhs.move_triangle,
									&rhs.move_triangle,
									changes.move_triangle,
								),
								(AttackType::Cross, &lhs.move_cross, &rhs.move_cross, changes.move_cross),
							] {
								if changed {
									if lhs_mv.name != rhs_mv.name {
										ui.label(format!("{attack} move name: {}", lhs_mv.name));
										ui.label(format!("{attack} move name: {}", rhs_mv.name));
										ui.end_row();
									}
									if lhs_mv.power != rhs_mv.power {
										ui.label(format!("{attack} move power: {}", lhs_mv.power));
										ui.label(format!("{attack} move power: {}", rhs_mv.power));
										ui.end_row();
									}
								}
							}
							if changes.cross_move_effect {
								self::display_cross_move_effect_opt(ui, &lhs.cross_move_effect);
								self::display_cross_move_effect_opt(ui, &rhs.cross_move_effect);
								ui.end_row();
							}
							if changes.effect_description.iter().any(|&changed| changed) {
								ui.label("Effect description");
								ui.label("Effect description");
								ui.end_row();
							}
							for (idx, &changed) in changes.effect_description.iter().enumerate() {
								if changed {
									ui.label(format!("\t#{}: {}", idx + 1, lhs.effect_description[idx]));
									ui.label(format!("\t#{}: {}", idx + 1, rhs.effect_description[idx]));
									ui.end_row();
								}
							}
							if changes.effect_arrow_color {
								let lhs = lhs.effect_arrow_color.map_or("None", ArrowColor::as_str);
								let rhs = rhs.effect_arrow_color.map_or("None", ArrowColor::as_str);
								ui.label(format!("Effect arrow color: {lhs}"));
								ui.label(format!("Effect arrow color: {rhs}"));
								ui.end_row();
							}
							for (idx, &changed) in changes.effect_conditions.iter().enumerate() {
								if changed {
									self::display_effect_condition_opt(ui, idx, &lhs.effect_conditions[idx]);
									self::display_effect_condition_opt(ui, idx, &rhs.effect_conditions[idx]);
									ui.end_row();
								}
							}
							for (idx, &changed) in changes.effects.iter().enumerate() {
								if changed {
									self::display_effect_opt(ui, idx, &lhs.effects[idx]);
									self::display_effect_opt(ui, idx, &rhs.effects[idx]);
									ui.end_row();
								}
							}
						},
						CardChanges::DifferentItem {
							lhs_id,
							rhs_id,
							changes,
						} => {
							let lhs = &lhs.cards[lhs_id].as_item().expect("Id wasn't for an item");
							let rhs = &rhs.cards[rhs_id].as_item().expect("Id wasn't for an item");

							ui.heading(name.as_str());
							ui.heading(name.as_str());
							ui.end_row();

							if changes.effect_description.iter().any(|&changed| changed) {
								ui.label("Effect description");
								ui.label("Effect description");
								ui.end_row();
							}
							for (idx, &changed) in changes.effect_description.iter().enumerate() {
								if changed {
									ui.label(format!("\t#{}: {}", idx + 1, lhs.effect_description[idx]));
									ui.label(format!("\t#{}: {}", idx + 1, rhs.effect_description[idx]));
									ui.end_row();
								}
							}
							if changes.effect_arrow_color {
								let lhs = lhs.effect_arrow_color.map_or("None", ArrowColor::as_str);
								let rhs = rhs.effect_arrow_color.map_or("None", ArrowColor::as_str);
								ui.label(format!("Effect arrow color: {lhs}"));
								ui.label(format!("Effect arrow color: {rhs}"));
								ui.end_row();
							}
							for (idx, &changed) in changes.effect_conditions.iter().enumerate() {
								if changed {
									self::display_effect_condition_opt(ui, idx, &lhs.effect_conditions[idx]);
									self::display_effect_condition_opt(ui, idx, &rhs.effect_conditions[idx]);
									ui.end_row();
								}
							}
							for (idx, &changed) in changes.effects.iter().enumerate() {
								if changed {
									self::display_effect_opt(ui, idx, &lhs.effects[idx]);
									self::display_effect_opt(ui, idx, &rhs.effects[idx]);
									ui.end_row();
								}
							}
						},
						CardChanges::DifferentDigivolve {
							lhs_id,
							rhs_id,
							changes,
						} => {
							let lhs = &lhs.cards[lhs_id].as_digivolve().expect("Id wasn't for a digivolve");
							let rhs = &rhs.cards[rhs_id].as_digivolve().expect("Id wasn't for a digivolve");

							ui.heading(name.as_str());
							ui.heading(name.as_str());
							ui.end_row();

							if changes.effect_description.iter().any(|&changed| changed) {
								ui.label("Effect description");
								ui.label("Effect description");
								ui.end_row();
							}
							for (idx, &changed) in changes.effect_description.iter().enumerate() {
								if changed {
									ui.label(format!("\t#{}: {}", idx + 1, lhs.effect_description[idx]));
									ui.label(format!("\t#{}: {}", idx + 1, rhs.effect_description[idx]));
									ui.end_row();
								}
							}
							if changes.effect {
								ui.label(format!("Effect: {}", lhs.effect));
								ui.label(format!("Effect: {}", rhs.effect));
								ui.end_row();
							}
						},
					}

					// TODO: Better way of doing a big separator?
					ui.separator();
					ui.separator();
					ui.end_row();
				}
			});
		});
	}
}

/// Displays an optional cross move effect
fn display_cross_move_effect_opt(ui: &mut egui::Ui, effect: &Option<CrossMoveEffect>) {
	ui.label(format!(
		"Cross move effect: {}",
		zutil::DisplayWrapper::new(|f| match effect {
			Some(effect) => write!(f, "{effect}"),
			None => write!(f, "None"),
		})
	));
}

/// Displays an optional effect condition
fn display_effect_condition_opt(ui: &mut egui::Ui, idx: usize, cond: &Option<EffectCondition>) {
	ui.label(format!(
		"Effect condition #{}: {}",
		idx + 1,
		zutil::DisplayWrapper::new(|f| match cond {
			// TODO: Properly display it
			Some(cond) => write!(f, "{cond:#?}"),
			None => write!(f, "None"),
		})
	));
}

/// Displays an optional effect
fn display_effect_opt(ui: &mut egui::Ui, idx: usize, effect: &Option<Effect>) {
	ui.label(format!(
		"Effect #{}: {}",
		idx + 1,
		zutil::DisplayWrapper::new(|f| match effect {
			// TODO: Properly display it
			Some(effect) => write!(f, "{effect:#?}"),
			None => write!(f, "None"),
		})
	));
}

/// Table changes
pub struct TableChanges {
	/// All card changes
	card_changes: BTreeMap<AsciiStrArr<0x14>, CardChanges>,
}

impl TableChanges {
	pub fn new(lhs: &CardTable, rhs: &CardTable) -> Self {
		// Then get the cards by their names on each loaded game
		let lhs_names = lhs
			.cards
			.iter()
			.enumerate()
			.map(|(idx, card)| (card.name(), (idx, card)))
			.collect::<BTreeMap<_, _>>();
		let rhs_names = rhs
			.cards
			.iter()
			.enumerate()
			.map(|(idx, card)| (card.name(), (idx, card)))
			.collect::<BTreeMap<_, _>>();

		let card_changes = KVParIter::new(&lhs_names, &rhs_names)
			.map(|(&&name, cards)| {
				let changes = match cards {
					ParIterValue::Both(&(lhs_id, lhs), &(rhs_id, rhs)) => match (lhs, rhs) {
						(lhs, rhs) if lhs == rhs => CardChanges::Equal { lhs_id, rhs_id },

						(Card::Digimon(lhs), Card::Digimon(rhs)) => {
							let mut changes = DigimonChanges::default();
							lhs.diff(rhs, &mut |diff: digimon::DiffKind| match diff {
								digimon::DiffKind::Name(..) => panic!("Name was different"),
								digimon::DiffKind::Speciality(..) => changes.speciality ^= true,
								digimon::DiffKind::Level(..) => changes.level ^= true,
								digimon::DiffKind::Hp(..) => changes.hp ^= true,
								digimon::DiffKind::DpCost(..) => changes.dp_cost ^= true,
								digimon::DiffKind::DpGive(..) => changes.dp_give ^= true,
								digimon::DiffKind::Move { attack, .. } => match attack {
									AttackType::Circle => changes.move_circle ^= true,
									AttackType::Triangle => changes.move_triangle ^= true,
									AttackType::Cross => changes.move_cross ^= true,
								},
								digimon::DiffKind::CrossMoveEffect(..) => changes.cross_move_effect ^= true,
								digimon::DiffKind::EffectDescription { idx, .. } => {
									changes.effect_description[idx] ^= true
								},
								digimon::DiffKind::EffectArrowColor(..) => changes.effect_arrow_color ^= true,
								digimon::DiffKind::EffectCondition { idx, .. } => {
									changes.effect_conditions[idx] ^= true
								},
								digimon::DiffKind::Effect { idx, .. } => changes.effects[idx] ^= true,
							});
							CardChanges::DifferentDigimon {
								lhs_id,
								rhs_id,
								changes,
							}
						},
						(Card::Item(lhs), Card::Item(rhs)) => {
							let mut changes = ItemChanges::default();
							lhs.diff(rhs, &mut |diff: item::DiffKind| match diff {
								item::DiffKind::Name(..) => panic!("Name was different"),
								item::DiffKind::EffectDescription { idx, .. } => {
									changes.effect_description[idx] ^= true
								},
								item::DiffKind::EffectArrowColor(..) => changes.effect_arrow_color ^= true,
								item::DiffKind::EffectCondition { idx, .. } => changes.effect_conditions[idx] ^= true,
								item::DiffKind::Effect { idx, .. } => changes.effects[idx] ^= true,
							});
							CardChanges::DifferentItem {
								lhs_id,
								rhs_id,
								changes,
							}
						},
						(Card::Digivolve(lhs), Card::Digivolve(rhs)) => {
							let mut changes = DigivolveChanges::default();
							lhs.diff(rhs, &mut |diff: digivolve::DiffKind| match diff {
								digivolve::DiffKind::Name(..) => panic!("Name was different"),
								digivolve::DiffKind::EffectDescription { idx, .. } => {
									changes.effect_description[idx] ^= true
								},
								digivolve::DiffKind::Effect(..) => changes.effect ^= true,
							});
							CardChanges::DifferentDigivolve {
								lhs_id,
								rhs_id,
								changes,
							}
						},
						_ => CardChanges::DifferentType { lhs_id, rhs_id },
					},
					ParIterValue::Left(&(id, _)) => CardChanges::OnlyInLhs { id },
					ParIterValue::Right(&(id, _)) => CardChanges::OnlyInRhs { id },
				};
				(name, changes)
			})
			.collect();

		Self { card_changes }
	}

	/// Returns all changes for each card
	pub fn card_changes(&self) -> &BTreeMap<AsciiStrArr<0x14>, CardChanges> {
		&self.card_changes
	}
}

/// Card changes
#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum CardChanges {
	/// Only in lhs
	OnlyInLhs { id: usize },

	/// Only in rhs
	OnlyInRhs { id: usize },

	/// Equal
	Equal { lhs_id: usize, rhs_id: usize },

	/// Different type
	DifferentType { lhs_id: usize, rhs_id: usize },

	/// Different digimon
	DifferentDigimon {
		lhs_id:  usize,
		rhs_id:  usize,
		changes: DigimonChanges,
	},

	/// Different item
	DifferentItem {
		lhs_id:  usize,
		rhs_id:  usize,
		changes: ItemChanges,
	},

	/// Different digivolve
	DifferentDigivolve {
		lhs_id:  usize,
		rhs_id:  usize,
		changes: DigivolveChanges,
	},
}

/// Digimon changes
#[derive(PartialEq, Eq, Clone, Copy, Default, Hash, Debug)]
pub struct DigimonChanges {
	pub speciality:         bool,
	pub level:              bool,
	pub hp:                 bool,
	pub dp_cost:            bool,
	pub dp_give:            bool,
	pub move_circle:        bool,
	pub move_triangle:      bool,
	pub move_cross:         bool,
	pub cross_move_effect:  bool,
	pub effect_description: [bool; 4],
	pub effect_arrow_color: bool,
	pub effect_conditions:  [bool; 2],
	pub effects:            [bool; 3],
}

/// Item changes
#[derive(PartialEq, Eq, Clone, Copy, Default, Hash, Debug)]
pub struct ItemChanges {
	pub effect_description: [bool; 4],
	pub effect_arrow_color: bool,
	pub effect_conditions:  [bool; 2],
	pub effects:            [bool; 3],
}

/// Digivolve changes
#[derive(PartialEq, Eq, Clone, Copy, Default, Hash, Debug)]
pub struct DigivolveChanges {
	pub effect_description: [bool; 4],
	pub effect:             bool,
}
