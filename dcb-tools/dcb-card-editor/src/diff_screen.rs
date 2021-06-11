//! Diff screen

// Imports
use crate::loaded_game::LoadedGame;
use anyhow::Context;
use dcb::card::{
	digimon, digivolve, item,
	property::{ArrowColor, CrossMoveEffect, Effect, EffectCondition},
	Card,
};
use dcb_util::{btree_map_par_iter::ParIterValue, BTreeMapParIter, StrContainsCaseInsensitive};
use eframe::egui::{self, Color32};
use std::{collections::BTreeMap, path::PathBuf};

/// Diff screen
pub struct DiffScreen {
	/// Other loaded game we're diffing against
	other_loaded_game: LoadedGame,

	/// Search
	search: String,
}

impl DiffScreen {
	/// Creates a new diff screen
	pub fn new(other_file_path: PathBuf) -> Result<Self, anyhow::Error> {
		// Load the other loaded game
		let other_loaded_game = LoadedGame::load(other_file_path).context("Unable to load other game")?;

		Ok(Self {
			other_loaded_game,
			search: String::new(),
		})
	}

	/// Displays this diff screen
	pub fn display(&mut self, ui: &mut egui::Ui, loaded_game: &LoadedGame) {
		let lhs = &loaded_game.card_table;
		let rhs = &self.other_loaded_game.card_table;

		// If the card table of both are equal, return
		if lhs == rhs {
			return;
		}

		// Else display the search
		let search = &mut self.search;
		ui.vertical(|ui| {
			ui.label("Search");
			ui.text_edit_singleline(search);
		});
		ui.separator();

		// Then get the cards by their names on each loaded game
		let lhs_names = lhs
			.cards
			.iter()
			.map(|card| (card.name(), card))
			.filter(|(name, _)| name.as_str().contains_case_insensitive(&*search))
			.collect::<BTreeMap<_, _>>();
		let rhs_names = rhs
			.cards
			.iter()
			.map(|card| (card.name(), card))
			.filter(|(name, _)| name.as_str().contains_case_insensitive(&*search))
			.collect::<BTreeMap<_, _>>();

		// Then display all differences
		egui::ScrollArea::auto_sized().show(ui, |ui| {
			egui::Grid::new("my_grid").striped(false).show(ui, |ui| {
				// Header
				ui.colored_label(Color32::from_rgb(255, 0, 0), "Loaded game");
				ui.colored_label(Color32::from_rgb(255, 0, 0), "Other game");
				ui.end_row();

				for (name, cards) in BTreeMapParIter::new(&lhs_names, &rhs_names) {
					match cards {
						ParIterValue::Both(lhs, rhs) => {
							// If they're equal, skip
							if lhs == rhs {
								continue;
							}
							ui.heading(name.as_str());
							ui.heading(name.as_str());
							ui.end_row();

							// If we already any difference in the effect descriptions
							let mut effect_descriptions_diff_found = false;

							// Check every property
							match (lhs, rhs) {
								(Card::Digimon(lhs), Card::Digimon(rhs)) => {
									lhs.diff(rhs, &mut |diff: digimon::DiffKind| match diff {
										digimon::DiffKind::Name(..) => {
											debug_assert!(false, "Name was different");
										},
										digimon::DiffKind::Speciality(lhs, rhs) => {
											ui.label(format!("Speciality: {lhs}"));
											ui.label(format!("Speciality: {rhs}"));
											ui.end_row();
										},
										digimon::DiffKind::Level(lhs, rhs) => {
											ui.label(format!("Level: {lhs}"));
											ui.label(format!("Level: {rhs}"));
											ui.end_row();
										},
										digimon::DiffKind::Hp(lhs, rhs) => {
											ui.label(format!("Hp: {lhs}"));
											ui.label(format!("Hp: {rhs}"));
											ui.end_row();
										},
										digimon::DiffKind::DpCost(lhs, rhs) => {
											ui.label(format!("DP: {lhs}"));
											ui.label(format!("DP: {rhs}"));
											ui.end_row();
										},
										digimon::DiffKind::DpGive(lhs, rhs) => {
											ui.label(format!("+P: {lhs}"));
											ui.label(format!("+P: {rhs}"));
											ui.end_row();
										},
										digimon::DiffKind::Move { attack, lhs, rhs } => {
											if lhs.name != rhs.name {
												ui.label(format!("{attack} move name: {}", lhs.name));
												ui.label(format!("{attack} move name: {}", rhs.name));
												ui.end_row();
											}
											if lhs.power != rhs.power {
												ui.label(format!("{attack} move power: {}", lhs.power));
												ui.label(format!("{attack} move power: {}", rhs.power));
												ui.end_row();
											}
										},
										digimon::DiffKind::CrossMoveEffect(lhs, rhs) => {
											self::display_cross_move_effect_opt(ui, &lhs);
											self::display_cross_move_effect_opt(ui, &rhs);
											ui.end_row();
										},
										digimon::DiffKind::EffectDescription { idx, lhs, rhs } => {
											// Print header if we hadn't found any differences yet
											if !effect_descriptions_diff_found {
												ui.label("Effect description");
												ui.label("Effect description");
												ui.end_row();
											}
											effect_descriptions_diff_found = true;

											ui.label(format!("\t#{}: {lhs}", idx + 1));
											ui.label(format!("\t#{}: {rhs}", idx + 1));
											ui.end_row();
										},
										digimon::DiffKind::EffectArrowColor(lhs, rhs) => {
											let lhs = lhs.map_or("None", ArrowColor::as_str);
											let rhs = rhs.map_or("None", ArrowColor::as_str);
											ui.label(format!("Effect arrow color: {lhs}"));
											ui.label(format!("Effect arrow color: {rhs}"));
											ui.end_row();
										},
										digimon::DiffKind::EffectCondition { idx, lhs, rhs } => {
											self::display_effect_condition_opt(ui, idx, &lhs);
											self::display_effect_condition_opt(ui, idx, &rhs);
											ui.end_row();
										},
										digimon::DiffKind::Effect { idx, lhs, rhs } => {
											self::display_effect_opt(ui, idx, lhs);
											self::display_effect_opt(ui, idx, rhs);
											ui.end_row();
										},
									});
								},
								(Card::Item(lhs), Card::Item(rhs)) => {
									lhs.diff(rhs, &mut |diff: item::DiffKind| match diff {
										item::DiffKind::Name(..) => {
											debug_assert!(false, "Name was different");
										},
										item::DiffKind::EffectDescription { idx, lhs, rhs } => {
											// Print header if we hadn't found any differences yet
											if !effect_descriptions_diff_found {
												ui.label("Effect description");
												ui.label("Effect description");
												ui.end_row();
											}
											effect_descriptions_diff_found = true;

											ui.label(format!("\t#{}: {lhs}", idx + 1));
											ui.label(format!("\t#{}: {rhs}", idx + 1));
											ui.end_row();
										},
										item::DiffKind::EffectArrowColor(lhs, rhs) => {
											let lhs = lhs.map_or("None", ArrowColor::as_str);
											let rhs = rhs.map_or("None", ArrowColor::as_str);
											ui.label(format!("Effect arrow color: {lhs}"));
											ui.label(format!("Effect arrow color: {rhs}"));
											ui.end_row();
										},
										item::DiffKind::EffectCondition { idx, lhs, rhs } => {
											self::display_effect_condition_opt(ui, idx, &lhs);
											self::display_effect_condition_opt(ui, idx, &rhs);
											ui.end_row();
										},
										item::DiffKind::Effect { idx, lhs, rhs } => {
											self::display_effect_opt(ui, idx, lhs);
											self::display_effect_opt(ui, idx, rhs);
											ui.end_row();
										},
									});
								},
								(Card::Digivolve(lhs), Card::Digivolve(rhs)) => {
									lhs.diff(rhs, &mut |diff: digivolve::DiffKind| match diff {
										digivolve::DiffKind::Name(..) => {
											debug_assert!(false, "Name was different");
										},
										digivolve::DiffKind::EffectDescription { idx, lhs, rhs } => {
											// Print header if we hadn't found any differences yet
											if !effect_descriptions_diff_found {
												ui.label("Effect description");
												ui.label("Effect description");
												ui.end_row();
											}
											effect_descriptions_diff_found = true;

											ui.label(format!("\t#{}: {lhs}", idx + 1));
											ui.label(format!("\t#{}: {rhs}", idx + 1));
											ui.end_row();
										},
										digivolve::DiffKind::Effect(lhs, rhs) => {
											ui.label(format!("Effect: {lhs}"));
											ui.label(format!("Effect: {rhs}"));
											ui.end_row();
										},
									});
								},
								// If they're different card types, simply emit their card types
								_ => {
									ui.label(lhs.ty().as_str());
									ui.label(rhs.ty().as_str());
									ui.end_row();
								},
							}
						},
						ParIterValue::Left(_) => {
							ui.heading(name.as_str());
							ui.heading("❌");
							ui.end_row();
						},
						ParIterValue::Right(_) => {
							ui.heading("❌");
							ui.heading(name.as_str());
							ui.end_row();
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
		dcb_util::DisplayWrapper::new(|f| match effect {
			Some(effect) => write!(f, "{effect}"),
			None => write!(f, "None"),
		})
	));
}

/// Displays an optional effect condition
fn display_effect_condition_opt(ui: &mut egui::Ui, idx: usize, cond: &Option<EffectCondition>) {
	ui.label(format!(
		"Cross move effect #{}: {}",
		idx + 1,
		dcb_util::DisplayWrapper::new(|f| match cond {
			// TODO: Properly display it
			Some(cond) => write!(f, "{cond:#?}"),
			None => write!(f, "None"),
		})
	));
}

/// Displays an optional effect
fn display_effect_opt(ui: &mut egui::Ui, idx: usize, effect: &Option<Effect>) {
	ui.label(format!(
		"Cross move effect #{}: {}",
		idx + 1,
		dcb_util::DisplayWrapper::new(|f| match effect {
			// TODO: Properly display it
			Some(effect) => write!(f, "{effect:#?}"),
			None => write!(f, "None"),
		})
	));
}
