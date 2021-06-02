//! Swap self

// Imports
use dcb::card::property::CardType;
use eframe::egui;

use crate::loaded_game::LoadedGame;

/// A swap self
pub struct SwapScreen {
	/// Card type
	card_type: CardType,

	/// Left idx
	lhs_idx: usize,

	/// Right idx
	rhs_idx: usize,
}

impl SwapScreen {
	/// Creates a new swap screen
	#[must_use]
	pub fn new(card_type: CardType, lhs_idx: usize, rhs_idx: usize) -> Self {
		Self {
			card_type,
			lhs_idx,
			rhs_idx,
		}
	}

	/// Displays this swap screen
	pub fn display(&mut self, ui: &mut egui::Ui, loaded_game: &mut LoadedGame) -> Results {
		let mut should_close = false;

		ui.horizontal(|ui| {
			ui.label("Card type");
			crate::render_card_type(ui, &mut self.card_type);
		});

		let range = 0..loaded_game.card_table.cards.len();
		if range.is_empty() {
			return Results { should_close: false };
		}

		self.lhs_idx = self.lhs_idx.clamp(range.start, range.end - 1);
		self.rhs_idx = self.rhs_idx.clamp(range.start, range.end - 1);
		let range = range.start..=(range.end - 1);

		ui.horizontal(|ui| {
			ui.label("Left");
			ui.add(egui::Slider::new(&mut self.lhs_idx, range.clone()));
		});
		ui.horizontal(|ui| {
			ui.label("Right");
			ui.add(egui::Slider::new(&mut self.rhs_idx, range));
		});
		if ui.button("Swap").clicked() {
			loaded_game.card_table.cards.swap(self.lhs_idx, self.rhs_idx);
			should_close = true;
		}

		Results { should_close }
	}
}

/// Display results
pub struct Results {
	/// If the self should be closed
	pub should_close: bool,
}
