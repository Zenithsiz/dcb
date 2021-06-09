//! Swap screen

// Imports
use dcb::CardTable;
use dcb_util::alert;
use eframe::egui;

/// Swap screen
pub struct SwapScreen {
	/// Left idx
	lhs_idx: usize,

	/// Right idx
	rhs_idx: usize,
}

impl SwapScreen {
	/// Creates a new swap screen
	#[must_use]
	pub fn new(lhs_idx: usize, rhs_idx: usize) -> Self {
		Self { lhs_idx, rhs_idx }
	}

	/// Displays this swap screen
	pub fn display(&mut self, ui: &mut egui::Ui, card_table: &mut CardTable) {
		// If there are no cards, return
		let range = 0..card_table.cards.len();
		if range.is_empty() {
			return;
		}

		// Else clamp our indexes and get our range as inclusive
		self.lhs_idx = self.lhs_idx.clamp(range.start, range.end - 1);
		self.rhs_idx = self.rhs_idx.clamp(range.start, range.end - 1);
		let range = range.start..=(range.end - 1);

		// Then draw the sliders for both cards
		ui.horizontal(|ui| {
			ui.label("Left");
			ui.add(egui::Slider::new(&mut self.lhs_idx, range.clone()));
		});
		ui.horizontal(|ui| {
			ui.label("Right");
			ui.add(egui::Slider::new(&mut self.rhs_idx, range));
		});

		// And check if they should be swapped
		if ui.button("Swap").clicked() {
			// Note: Cannot panic, as we clamp the indexes to their range
			card_table.cards.swap(self.lhs_idx, self.rhs_idx);

			// Note: Swapped because we just swapped them, but we want to display
			//       the previous names
			let rhs = &card_table.cards[self.lhs_idx];
			let lhs = &card_table.cards[self.rhs_idx];

			alert::info!(
				"Successfully swapped {} ({}) and {} ({})",
				lhs.name(),
				self.lhs_idx,
				rhs.name(),
				self.rhs_idx
			);
		}
	}
}
