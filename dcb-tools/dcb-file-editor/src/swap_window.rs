//! Swap window

// Imports
use crate::GameFile;
use anyhow::Context;
use dcb_io::game_file::Path;
use dcb_util::{alert, task};
use eframe::egui;
use std::{mem, sync::Arc};

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
	pub fn display(&mut self, ctx: &egui::CtxRef, game_file: &Arc<GameFile>) {
		egui::Window::new("Swap screen").show(ctx, |ui| {
			ui.horizontal(|ui| {
				ui.label(self.first.as_str().unwrap_or("None"));
				if ui.button(self.first.button_label()).clicked() {
					self.first.toggle();
				}
			});
			ui.horizontal(|ui| {
				ui.label(self.second.as_str().unwrap_or("None"));
				if ui.button(self.second.button_label()).clicked() {
					self.second.toggle();
				}
			});

			if ui.button("Swap").clicked() {
				match (&self.first, &self.second) {
					// If we got both, spawn the task to swap the files
					(SwapFileStatus::Set(lhs), SwapFileStatus::Set(rhs)) => {
						let lhs = lhs.clone();
						let rhs = rhs.clone();
						let game_file = Arc::clone(game_file);
						task::spawn(move || {
							// TODO: Use `PathBuf` once `dcb_io` supports it
							let lhs = Path::from_ascii(&lhs).expect("Lhs path wasn't valid");
							let rhs = Path::from_ascii(&rhs).expect("Rhs path wasn't valid");

							// Try to swap them and reload the game file
							let res: Result<_, anyhow::Error> = try {
								game_file.with_game_file(|mut game_file| {
									game_file.swap_files(lhs, rhs).context("Unable to swap files")
								})?;
								game_file.reload().context("Unable to reload the game")?;
							};

							// Then alert the user
							match res {
								Ok(()) => alert::info("Successfully swapped {lhs} with {rhs}!"),
								Err(err) => alert::error(&format!("Unable to swap files: {:?}", err)),
							}
						});
					},
					_ => alert::warn("You must set both files before swapping"),
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
