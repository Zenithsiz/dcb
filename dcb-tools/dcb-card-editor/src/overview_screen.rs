//! Overview screen

use std::collections::HashMap;

// Imports
use crate::loaded_game::LoadedGame;
use dcb::card::property::Speciality;
use eframe::egui;
use strum::IntoEnumIterator;


/// Overview screen
pub struct OverviewScreen {
	/// Total cards
	total_cards: usize,

	/// Number of cards per speciality
	cards_per_speciality: HashMap<Speciality, usize>,

	/// Total digimons
	total_digimons: usize,
}

impl OverviewScreen {
	/// Creates a new screen
	pub fn new(loaded_game: &LoadedGame) -> Self {
		let card_table = loaded_game.card_table();
		let total_digimons = card_table.digimons.len();
		let total_cards = card_table.digimons.len() + card_table.items.len() + card_table.digivolves.len();

		let cards_per_speciality = Speciality::iter()
			.map(|speciality| {
				(
					speciality,
					card_table
						.digimons
						.iter()
						.filter(|digimon| digimon.speciality == speciality)
						.count(),
				)
			})
			.collect();

		Self {
			total_cards,
			cards_per_speciality,
			total_digimons,
		}
	}

	/// Reloads this screen
	pub fn reload(&mut self, loaded_game: &LoadedGame) {
		*self = Self::new(loaded_game);
	}

	/// Displays this swap screen
	pub fn display(&mut self, ui: &mut egui::Ui, loaded_game: &LoadedGame) {
		let Self {
			total_cards,
			cards_per_speciality,
			total_digimons,
		} = &*self;


		ui.horizontal(|ui| {
			ui.label("Total cards");
			ui.label(format!("{total_cards}"));
		});
		egui::Grid::new(self as *const _).striped(true).show(ui, |ui| {
			ui.label("Speciality");
			ui.label("Number of cards");
			ui.label("%");
			ui.end_row();

			for speciality in Speciality::iter() {
				let cards_len = cards_per_speciality[&speciality];

				ui.label(speciality.as_str());
				ui.label(format!("{cards_len}"));
				ui.label(format!("{:.2}%", 100.0 * cards_len as f32 / *total_digimons as f32));

				ui.end_row()
			}

			ui.label("Total");
			ui.label(format!(
				"{}",
				cards_per_speciality.iter().map(|(_, len)| len).sum::<usize>()
			));
			ui.label("100.00%");
			ui.end_row();
		});

		// Reload if the users wants to
		if ui.button("Reload").clicked() {
			self.reload(loaded_game);
		}
	}
}
