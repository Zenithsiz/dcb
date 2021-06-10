//! Diff screen

// Imports
use eframe::egui;

/// Diff screen
pub struct DiffScreen {}

impl DiffScreen {
	/// Creates a new diff screen
	pub fn new() -> Self {
		Self {}
	}

	/// Displays this diff screen
	pub fn display(&mut self, ui: &mut egui::Ui) {
		let Self {} = &*self;

		ui.horizontal(|ui| {
			ui.label("Diff");
		});
	}
}
