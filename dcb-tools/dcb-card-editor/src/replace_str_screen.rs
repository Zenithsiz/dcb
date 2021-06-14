//! Replace string screen

// Imports
use dcb::CardTable;
use dcb_util::{alert, ascii_str_arr::AsciiChar, AsciiStrArr};
use eframe::egui;

/// Replace string screen
pub struct ReplaceStrScreen {
	/// String to search
	search: String,

	/// Replace
	replace: String,

	// TODO: Use case sensitive
	/*
	/// Case sensitive
	case_sensitive: bool,
	*/
	/// Names
	names: bool,

	/// Effect descriptions
	effect_descriptions: bool,
}

impl ReplaceStrScreen {
	/// Creates a new swap screen
	#[must_use]
	pub fn new() -> Self {
		Self {
			search:              String::new(),
			replace:             String::new(),
			names:               true,
			effect_descriptions: true,
		}
	}

	/// Displays this swap screen
	pub fn display(&mut self, ui: &mut egui::Ui, card_table: &mut CardTable) {
		ui.horizontal(|ui| {
			ui.label("Search");
			ui.text_edit_singleline(&mut self.search);
		});

		ui.horizontal(|ui| {
			ui.label("Replace");
			ui.text_edit_singleline(&mut self.replace);
		});

		ui.checkbox(&mut self.names, "Names");
		ui.checkbox(&mut self.effect_descriptions, "Effect descriptions");

		if ui.button("Replace").clicked() {
			for card in &mut card_table.cards {
				if self.names {
					self.replace_str(card.name_mut());
				}

				if self.effect_descriptions {
					for desc in card.effect_description_mut() {
						self.replace_str(desc);
					}
				}
			}
		}
	}

	// TODO: Redo this, pretty bad
	fn replace_str(&self, s: &mut AsciiStrArr<0x14>) {
		if let Some(idx) = s.as_str().find(&self.search) {
			s.drain_range(idx..(idx + self.search.len()));
			for ch in self.replace.chars().rev() {
				let ch = match AsciiChar::from_ascii(ch) {
					Ok(ch) => ch,
					Err(_) => {
						alert::error!("{ch} isn't a valid ascii character");
						continue;
					},
				};

				if s.insert(idx, ch).is_err() {
					alert::error!("String to replace was too large");
					break;
				}
			}
		}
	}
}
