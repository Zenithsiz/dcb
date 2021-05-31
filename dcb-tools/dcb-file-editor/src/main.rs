//! Card editor

// Features
#![feature(
	array_map,
	with_options,
	format_args_capture,
	once_cell,
	never_type,
	try_blocks,
	hash_drain_filter
)]

// Modules
pub mod loaded_game;
pub mod preview_panel;
pub mod swap_window;
pub mod tree;

// Imports
use anyhow::Context;
use dcb_cdrom_xa::CdRomCursor;
use dcb_io::GameFile;
use eframe::{egui, epi, NativeOptions};
use loaded_game::LoadedGame;
use native_dialog::{FileDialog, MessageDialog, MessageType};
use preview_panel::PreviewPanel;
use std::{fs, io::Write, path::PathBuf};
use swap_window::SwapWindow;

fn main() {
	// Initialize the logger
	simplelog::TermLogger::init(
		log::LevelFilter::Debug,
		simplelog::Config::default(),
		simplelog::TerminalMode::Stderr,
		simplelog::ColorChoice::Auto,
	)
	.expect("Unable to initialize logger");

	// Crate the app and run it
	let app = FileEditor::default();
	eframe::run_native(Box::new(app), NativeOptions::default());
}

pub struct FileEditor {
	/// File path
	file_path: Option<PathBuf>,

	/// Loaded game
	loaded_game: Option<LoadedGame>,

	/// File search
	file_search: String,

	/// Swap window
	swap_window: Option<SwapWindow>,

	/// Preview panel
	preview_panel: Option<PreviewPanel>,
}

impl Default for FileEditor {
	fn default() -> Self {
		Self {
			file_path:     None,
			loaded_game:   None,
			file_search:   String::new(),
			swap_window:   None,
			preview_panel: None,
		}
	}
}

impl epi::App for FileEditor {
	fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
		let Self {
			file_path,
			loaded_game,
			file_search,
			swap_window,
			preview_panel,
		} = self;

		// Top panel
		egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
			egui::menu::bar(ui, |ui| {
				egui::menu::menu(ui, "File", |ui| {
					// On open, ask the user and open the file
					if ui.button("Open").clicked() {
						let cur_dir_path = std::env::current_dir().expect("Unable to get current directory path");
						*file_path = FileDialog::new()
							.set_location(&cur_dir_path)
							.add_filter("Game file", &["bin"])
							.show_open_single_file()
							.expect("Unable to ask user for file");

						// Then open the file
						if let Some(file_path) = file_path {
							let res: Result<_, anyhow::Error> = try {
								let file = fs::File::with_options()
									.read(true)
									.write(true)
									.open(file_path)
									.context("Unable to open file")?;
								let cdrom = CdRomCursor::new(file);
								let game_file = GameFile::new(cdrom);

								*loaded_game = Some(LoadedGame::new(game_file).context("Unable to load game")?);
							};

							if let Err(err) = res {
								MessageDialog::new()
									.set_text(&format!("Unable to open file: {:?}", err))
									.set_type(MessageType::Error)
									.show_alert()
									.expect("Unable to alert user");
							}
						}
					}

					if ui.button("Quit").clicked() {
						frame.quit();
					}
				});

				egui::menu::menu(ui, "Edit", |ui| {
					if loaded_game.is_some() && ui.button("Swap").clicked() {
						*swap_window = Some(SwapWindow::default())
					}
				});
			});
		});

		egui::SidePanel::left("side_panel").show(ctx, |ui| {
			ui.heading("File list");

			ui.vertical(|ui| {
				ui.label("Search");
				ui.text_edit_singleline(file_search);
			});

			// If we have a loaded game, display all files
			if let Some(loaded_game) = loaded_game.as_mut() {
				loaded_game.display(frame, ui, file_search, swap_window, preview_panel);
			}
		});

		if let Some(preview_panel) = preview_panel {
			preview_panel.display(ctx);
		}

		if let (Some(swap_window), Some(loaded_game)) = (swap_window, loaded_game) {
			swap_window.display(ctx, loaded_game)
		}
	}

	fn on_exit(&mut self) {
		// Flush the file if we have it
		if let Some(loaded_game) = &mut self.loaded_game {
			match loaded_game.game_file_mut().cdrom().flush() {
				Ok(()) => (),
				Err(err) => MessageDialog::new()
					.set_text(&format!("Unable to flush file tod isk: {:?}", err))
					.set_type(MessageType::Error)
					.show_alert()
					.expect("Unable to alert user"),
			}
		}
	}

	fn name(&self) -> &str {
		"Dcb file editor"
	}
}
