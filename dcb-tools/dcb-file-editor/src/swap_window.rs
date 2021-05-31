//! Swap window

// Imports
use crate::LoadedGame;
use anyhow::Context;
use dcb_io::game_file::Path;
use eframe::egui;
use native_dialog::{MessageDialog, MessageType};
use std::mem;

/// Swap window
#[derive(PartialEq, Clone, Default)]
pub struct SwapWindow {
	/// First file
	first: SwapFileStatus,

	/// Second file
	second: SwapFileStatus,
}

impl SwapWindow {
	/// On file click
	pub fn on_file_click(&mut self, path: &str) {
		if self.first.is_setting() {
			self.first = SwapFileStatus::Set(path.to_owned());
		}
		if self.second.is_setting() {
			self.second = SwapFileStatus::Set(path.to_owned());
		}
	}

	/// Displays this swap window
	pub fn display(&mut self, ctx: &egui::CtxRef, loaded_game: &mut LoadedGame) {
		egui::Window::new("Swap screen").show(ctx, |ui| {
			ui.horizontal(|ui| {
				ui.label(self.first.as_str().unwrap_or("None"));
				if ui.button(self.second.button_label()).clicked() {
					self.first.toggle();
				}
			});
			ui.horizontal(|ui| {
				ui.label(self.second.as_str().unwrap_or("None"));
				if ui.button(self.first.button_label()).clicked() {
					self.second.toggle();
				}
			});

			if ui.button("Swap").clicked() {
				match (&self.first, &self.second) {
					(SwapFileStatus::Set(lhs), SwapFileStatus::Set(rhs)) => {
						let lhs = Path::from_ascii(lhs).expect("Lhs path wasn't valid");
						let rhs = Path::from_ascii(rhs).expect("Rhs path wasn't valid");

						let res: Result<_, anyhow::Error> = try {
							loaded_game
								.game_file
								.swap_files(lhs, rhs)
								.context("Unable to swap files")?;

							loaded_game.reload().context("Unable to reload the game")?;
						};

						match res {
							Ok(()) => {
								MessageDialog::new()
									.set_text("Successfully swapped!")
									.set_type(MessageType::Info)
									.show_alert()
									.expect("Unable to alert user");
							},
							Err(err) => MessageDialog::new()
								.set_text(&format!("Unable to swap files: {:?}", err))
								.set_type(MessageType::Error)
								.show_alert()
								.expect("Unable to alert user"),
						}
					},
					_ => MessageDialog::new()
						.set_text("You must set both files before swapping")
						.set_type(MessageType::Warning)
						.show_alert()
						.expect("Unable to alert user"),
				}
			}
		});
	}
}

/// File setting status
#[derive(PartialEq, Clone)]
pub enum SwapFileStatus {
	/// Unset
	Unset,

	/// Setting
	Setting(Option<String>),

	/// Set
	Set(String),
}

impl SwapFileStatus {
	/// Returns the button label for this status
	fn button_label(&mut self) -> &str {
		match self.is_setting() {
			true => "...",
			false => "Set",
		}
	}
}

impl SwapFileStatus {
	/// Toggles the current setting
	pub fn toggle(&mut self) {
		*self = match mem::take(self) {
			Self::Unset => Self::Setting(None),
			Self::Setting(s) => match s {
				Some(s) => Self::Set(s),
				None => Self::Unset,
			},
			Self::Set(s) => Self::Setting(Some(s)),
		};
	}

	/// Returns this status as a string
	pub fn as_str(&self) -> Option<&str> {
		match self {
			Self::Setting(s) => s.as_deref(),
			Self::Set(s) => Some(s),
			_ => None,
		}
	}

	/// Returns `true` if the swap_file_status is [`Setting`].
	pub fn is_setting(&self) -> bool {
		matches!(self, Self::Setting(..))
	}

	/// Returns `true` if the swap_file_status is [`Set`].
	pub fn is_set(&self) -> bool {
		matches!(self, Self::Set(..))
	}

	/// Returns `true` if the swap_file_status is [`Unset`].
	pub fn is_unset(&self) -> bool {
		matches!(self, Self::Unset)
	}
}

impl Default for SwapFileStatus {
	fn default() -> Self {
		Self::Unset
	}
}
