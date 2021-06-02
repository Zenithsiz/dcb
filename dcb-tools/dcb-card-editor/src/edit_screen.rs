//! Edit screen

// Imports
use crate::loaded_game::LoadedGame;
use dcb::card::Card;
use eframe::egui;

/// An edit screen
pub struct EditScreen {
	/// Currently selected card
	card_idx: usize,
}

impl EditScreen {
	/// Creates a new edit screen from it's card index
	#[must_use]
	pub fn new(card_idx: usize) -> Self {
		Self { card_idx }
	}

	/// Returns the card index of this screen
	#[must_use]
	pub fn card_idx(&self) -> usize {
		self.card_idx
	}

	/// Displays all edit screens
	pub fn display_all(screens: &mut Vec<Self>, ctx: &egui::CtxRef, loaded_game: &mut LoadedGame) {
		let screen_width = ctx.available_rect().width() / (screens.len() as f32);
		for screen in screens {
			let card = &mut loaded_game.card_table.cards[screen.card_idx];

			egui::SidePanel::left(screen as *const _)
				.min_width(screen_width)
				.max_width(screen_width)
				.show(ctx, |ui| {
					// Header for the card
					ui.vertical(|ui| {
						ui.heading(card.name().as_str());
						ui.label(match card {
							Card::Digimon(_) => "Digimon",
							Card::Item(_) => "Item",
							Card::Digivolve(_) => "Digivolve",
						});
						ui.separator();
					});

					egui::ScrollArea::auto_sized().show(ui, |ui| {
						crate::render_card(ui, card);
					});
				});
		}
	}
}
