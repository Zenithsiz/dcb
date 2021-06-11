//! Diff screen

// Imports
use crate::loaded_game::LoadedGame;
use anyhow::Context;
use dcb::card::{
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

							// Check every property
							match (lhs, rhs) {
								(Card::Digimon(lhs), Card::Digimon(rhs)) => {
									if lhs.speciality != rhs.speciality {
										ui.label(format!("Speciality: {}", lhs.speciality.as_str()));
										ui.label(format!("Speciality: {}", rhs.speciality.as_str()));
										ui.end_row();
									}
									if lhs.level != rhs.level {
										ui.label(format!("Level: {}", lhs.level.as_str()));
										ui.label(format!("Level: {}", rhs.level.as_str()));
										ui.end_row();
									}
									if lhs.hp != rhs.hp {
										ui.label(format!("Hp: {}", lhs.hp));
										ui.label(format!("Hp: {}", rhs.hp));
										ui.end_row();
									}
									if lhs.dp_cost != rhs.dp_cost {
										ui.label(format!("DP: {}", lhs.dp_cost));
										ui.label(format!("DP: {}", rhs.dp_cost));
										ui.end_row();
									}
									if lhs.dp_give != rhs.dp_give {
										ui.label(format!("+P: {}", lhs.dp_give));
										ui.label(format!("+P: {}", rhs.dp_give));
										ui.end_row();
									}
									for (mv_name, lhs_mv, rhs_mv) in std::array::IntoIter::new([
										("Circle", &lhs.move_circle, &rhs.move_circle),
										("Triangle", &lhs.move_triangle, &rhs.move_triangle),
										("Cross", &lhs.move_cross, &rhs.move_cross),
									]) {
										if lhs_mv.name != rhs_mv.name {
											ui.label(format!("{} move name: {}", mv_name, lhs_mv.name));
											ui.label(format!("{} move name: {}", mv_name, rhs_mv.name));
											ui.end_row();
										}
										if lhs_mv.power != rhs_mv.power {
											ui.label(format!("{} move power: {}", mv_name, lhs_mv.power));
											ui.label(format!("{} move power: {}", mv_name, rhs_mv.power));
											ui.end_row();
										}
									}
									if lhs.cross_move_effect != rhs.cross_move_effect {
										self::display_cross_move_effect_opt(ui, &lhs.cross_move_effect);
										self::display_cross_move_effect_opt(ui, &rhs.cross_move_effect);
										ui.end_row();
									}
									if lhs.effect_description != rhs.effect_description {
										ui.label("Effect description");
										ui.label("Effect description");
										ui.end_row();

										for (idx, (lhs_desc, rhs_desc)) in
											(lhs.effect_description.zip(rhs.effect_description)).iter().enumerate()
										{
											if lhs_desc == rhs_desc {
												continue;
											}

											ui.label(format!("\t#{}: {}", idx + 1, lhs_desc.as_str()));
											ui.label(format!("\t#{}: {}", idx + 1, rhs_desc.as_str()));
											ui.end_row();
										}
									}
									if lhs.effect_arrow_color != rhs.effect_arrow_color {
										ui.label(format!(
											"Effect arrow color: {}",
											lhs.effect_arrow_color.map_or("None", ArrowColor::as_str)
										));
										ui.label(format!(
											"Effect arrow color: {}",
											rhs.effect_arrow_color.map_or("None", ArrowColor::as_str)
										));
										ui.end_row();
									}
									for (idx, (lhs_cond, rhs_cond)) in
										(lhs.effect_conditions.zip(rhs.effect_conditions)).iter().enumerate()
									{
										if lhs_cond != rhs_cond {
											self::display_effect_condition_opt(ui, idx, lhs_cond);
											self::display_effect_condition_opt(ui, idx, rhs_cond);
											ui.end_row();
										}
									}
									for (idx, (lhs_effect, rhs_effect)) in
										(lhs.effects.zip(rhs.effects)).iter().enumerate()
									{
										if lhs_effect != rhs_effect {
											self::display_effect_opt(ui, idx, lhs_effect);
											self::display_effect_opt(ui, idx, rhs_effect);
											ui.end_row();
										}
									}
								},
								(Card::Item(lhs), Card::Item(rhs)) => {
									if lhs.effect_description != rhs.effect_description {
										ui.label("Effect description");
										ui.label("Effect description");
										ui.end_row();

										for (idx, (lhs_desc, rhs_desc)) in
											(lhs.effect_description.zip(rhs.effect_description)).iter().enumerate()
										{
											if lhs_desc == rhs_desc {
												continue;
											}

											ui.label(format!("\t#{}: {}", idx + 1, lhs_desc.as_str()));
											ui.label(format!("\t#{}: {}", idx + 1, rhs_desc.as_str()));
											ui.end_row();
										}
									}
									if lhs.effect_arrow_color != rhs.effect_arrow_color {
										ui.label(format!(
											"Effect arrow color: {}",
											lhs.effect_arrow_color.map_or("None", ArrowColor::as_str)
										));
										ui.label(format!(
											"Effect arrow color: {}",
											rhs.effect_arrow_color.map_or("None", ArrowColor::as_str)
										));
										ui.end_row();
									}
									for (idx, (lhs_cond, rhs_cond)) in
										(lhs.effect_conditions.zip(rhs.effect_conditions)).iter().enumerate()
									{
										if lhs_cond != rhs_cond {
											self::display_effect_condition_opt(ui, idx, lhs_cond);
											self::display_effect_condition_opt(ui, idx, rhs_cond);
											ui.end_row();
										}
									}
									for (idx, (lhs_effect, rhs_effect)) in
										(lhs.effects.zip(rhs.effects)).iter().enumerate()
									{
										if lhs_effect != rhs_effect {
											self::display_effect_opt(ui, idx, lhs_effect);
											self::display_effect_opt(ui, idx, rhs_effect);
											ui.end_row();
										}
									}
								},
								(Card::Digivolve(lhs), Card::Digivolve(rhs)) => {
									if lhs.effect_description != rhs.effect_description {
										ui.label("Effect description");
										ui.label("Effect description");
										ui.end_row();

										for (idx, (lhs_desc, rhs_desc)) in
											(lhs.effect_description.zip(rhs.effect_description)).iter().enumerate()
										{
											if lhs_desc == rhs_desc {
												continue;
											}

											ui.label(format!("\t#{}: {}", idx + 1, lhs_desc.as_str()));
											ui.label(format!("\t#{}: {}", idx + 1, rhs_desc.as_str()));
											ui.end_row();
										}
									}
									if lhs.effect != rhs.effect {
										ui.label(format!("Effect: {}", lhs.effect));
										ui.label(format!("Effect: {}", rhs.effect));
										ui.end_row();
									}
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
